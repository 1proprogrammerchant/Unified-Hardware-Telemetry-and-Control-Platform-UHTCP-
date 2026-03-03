#ifndef SENSOR_HPP
#define SENSOR_HPP

#include "Device.hpp"
#include <mutex>
#include <condition_variable>
#include <thread>
#include <atomic>
#include <string>
#include <chrono>

class Sensor : public Device {
public:
    Sensor();
    Sensor(const std::string &id_, const std::string &name_);
    ~Sensor();

    // Device overrides
    void synchronize() override;
    void apply_control() override;

    // Lifecycle
    void start_sampling(unsigned int interval_ms = 100);
    void stop_sampling();

    // Read the last filtered measurement
    double read_measurement() const;

    // Explicit immediate read (blocking, retries)
    bool read_now(double &out_value);

    // Calibration helpers
    void calibrate(double offset);

    // Error handling / recovery
    void handleError() override;
    void recover() override;

    // Diagnostics
    unsigned int failure_count() const;
    unsigned int success_count() const;
    unsigned long long last_update_ms() const;

    // Configuration
    void set_source_url(const std::string &url);
    void set_source_file(const std::string &path);

private:
    void sampling_loop();
    bool fetch_value(double &out);
    bool fetch_remote(const std::string &url, double &out);
    bool fetch_file(const std::string &path, double &out);

    // internal
    mutable std::mutex mtx;
    std::condition_variable cv;
    std::thread sampler;
    std::atomic<bool> running{false};

    // data
    double raw_value = 0.0;
    double filtered = 0.0; // EMA
    double calibration_offset = 0.0;
    double ema_alpha = 0.2;

    // sampling config
    unsigned int interval_ms = 100;
    unsigned int debounce_count = 0;
    unsigned int debounce_threshold = 3;

    // failure tracking + backoff
    unsigned int fails = 0;
    unsigned int successes = 0;
    unsigned int max_failures_before_recover = 5;
    unsigned int base_backoff_ms = 50;
    std::chrono::steady_clock::time_point backoff_until = std::chrono::steady_clock::now();

    // source specifiers (prefer remote if set)
    std::string source_url;
    std::string source_file;

    // last update time (ms since epoch)
    std::atomic<unsigned long long> last_update{0};
};

#endif
