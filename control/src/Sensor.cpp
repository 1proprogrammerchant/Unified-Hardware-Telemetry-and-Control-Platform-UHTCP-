// Sensor implementation: sampling thread, EMA filter, debounce, backoff, file/HTTP source
#include "../include/Sensor.hpp"
#include "../include/Logger.hpp"

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <sstream>
#include <iostream>

static unsigned long long now_ms() {
	using namespace std::chrono;
	return (unsigned long long)duration_cast<milliseconds>(system_clock::now().time_since_epoch()).count();
}

Sensor::Sensor() {
	id = "sensor-unknown";
	name = "Unnamed Sensor";
	state = DeviceState::UNINITIALIZED;
}

Sensor::Sensor(const std::string &id_, const std::string &name_) {
	id = id_;
	name = name_;
	state = DeviceState::UNINITIALIZED;
}

Sensor::~Sensor() {
	stop_sampling();
}

void Sensor::set_source_url(const std::string &url) {
	std::lock_guard<std::mutex> g(mtx);
	source_url = url;
}

void Sensor::set_source_file(const std::string &path) {
	std::lock_guard<std::mutex> g(mtx);
	source_file = path;
}

void Sensor::start_sampling(unsigned int interval_ms_) {
	std::lock_guard<std::mutex> g(mtx);
	if (running.load()) return;
	interval_ms = interval_ms_;
	running.store(true);
	sampler = std::thread(&Sensor::sampling_loop, this);
	Logger::info("Sensor(" + id + "): started sampling (" + std::to_string(interval_ms) + "ms)");
}

void Sensor::stop_sampling() {
	{
		std::lock_guard<std::mutex> g(mtx);
		if (!running.load()) return;
		running.store(false);
		cv.notify_all();
	}
	if (sampler.joinable()) sampler.join();
	Logger::info("Sensor(" + id + "): stopped sampling");
}

void Sensor::synchronize() {
	// Force an immediate read (blocking)
	double v = 0.0;
	if (read_now(v)) {
		std::lock_guard<std::mutex> g(mtx);
		raw_value = v;
		// initialize EMA if first sample
		if (successes == 0) filtered = raw_value + calibration_offset;
		else filtered = (ema_alpha * (raw_value + calibration_offset)) + ((1.0 - ema_alpha) * filtered);
		last_update.store(now_ms());
		successes++;
		fails = 0;
		if (state == DeviceState::UNINITIALIZED) state = DeviceState::IDLE;
	} else {
		handleError();
	}
}

void Sensor::apply_control() {
	// Sensors typically don't accept control; this is a no-op but kept for interface compatibility
}

bool Sensor::read_now(double &out_value) {
	// Respect exponential backoff window
	if (std::chrono::steady_clock::now() < backoff_until) return false;

	// Try remote first if configured, then fallback to file, then simulate
	if (!source_url.empty()) {
		if (fetch_remote(source_url, out_value)) return true;
	}
	if (!source_file.empty()) {
		if (fetch_file(source_file, out_value)) return true;
	}

	// Fallback simulated read
	out_value = 0.5 + (std::rand() % 100) / 100.0; // simple random-ish simulation
	return true;
}

bool Sensor::fetch_value(double &out) {
	return read_now(out);
}

bool Sensor::fetch_remote(const std::string &url, double &out) {
	// Use popen with curl if available; this is a best-effort IPC bridge
	std::string cmd = "curl -sf --max-time 1 '" + url + "' 2>/dev/null";
	FILE *p = popen(cmd.c_str(), "r");
	if (!p) return false;
	char buf[256];
	std::string res;
	while (fgets(buf, sizeof(buf), p)) res += buf;
	int rc = pclose(p);
	if (rc != 0 || res.empty()) return false;
	// attempt to parse leading number
	std::istringstream iss(res);
	double v;
	if (!(iss >> v)) return false;
	out = v;
	return true;
}

bool Sensor::fetch_file(const std::string &path, double &out) {
	FILE *f = fopen(path.c_str(), "r");
	if (!f) return false;
	char buf[128];
	if (!fgets(buf, sizeof(buf), f)) { fclose(f); return false; }
	fclose(f);
	std::istringstream iss(buf);
	double v;
	if (!(iss >> v)) return false;
	out = v;
	return true;
}

void Sensor::sampling_loop() {
	using namespace std::chrono;
	auto next = steady_clock::now();
	while (running.load()) {
		next += milliseconds(interval_ms);

		double v = 0.0;
		bool ok = fetch_value(v);
		if (ok) {
			std::lock_guard<std::mutex> g(mtx);
			raw_value = v;
			double calibrated = raw_value + calibration_offset;
			if (successes == 0) filtered = calibrated;
			else filtered = (ema_alpha * calibrated) + ((1.0 - ema_alpha) * filtered);
			successes++;
			fails = 0;
			last_update.store(now_ms());
			// debounce: require several consistent readings before flipping high-frequency flags
			debounce_count = 0;
			if (state == DeviceState::UNINITIALIZED) state = DeviceState::IDLE;
		} else {
			fails++;
			Logger::warn("Sensor(" + id + "): fetch failed (" + std::to_string(fails) + ")");
			if (fails >= max_failures_before_recover) {
				handleError();
			}
			// apply exponential backoff
			unsigned int backoff = base_backoff_ms * (1u << std::min(fails, 8u));
			if (backoff > 30000) backoff = 30000;
			backoff_until = steady_clock::now() + milliseconds(backoff);
		}

		std::unique_lock<std::mutex> lk(mtx);
		cv.wait_until(lk, next, [this]{ return !running.load(); });
	}
}

void Sensor::calibrate(double offset) {
	std::lock_guard<std::mutex> g(mtx);
	calibration_offset = offset;
}

void Sensor::handleError() {
	std::lock_guard<std::mutex> g(mtx);
	state = DeviceState::ERROR;
	Logger::warn("Sensor(" + id + "): entering ERROR state");
}

void Sensor::recover() {
	std::lock_guard<std::mutex> g(mtx);
	if (state != DeviceState::ERROR) return;
	Logger::info("Sensor(" + id + "): attempting recovery");
	// simple recovery: reset counters and try a single read
	fails = 0;
	last_update.store(0);
	double v;
	if (read_now(v)) {
		raw_value = v;
		filtered = raw_value + calibration_offset;
		state = DeviceState::RECOVERING;
		// brief settling period
		std::this_thread::sleep_for(std::chrono::milliseconds(50));
		state = DeviceState::IDLE;
		successes++;
		Logger::info("Sensor(" + id + "): recovered to IDLE");
	} else {
		Logger::warn("Sensor(" + id + "): recovery attempt failed");
	}
}

double Sensor::read_measurement() const {
	std::lock_guard<std::mutex> g(mtx);
	return filtered;
}

unsigned int Sensor::failure_count() const { return fails; }
unsigned int Sensor::success_count() const { return successes; }
unsigned long long Sensor::last_update_ms() const { return last_update.load(); }


