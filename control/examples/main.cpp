#include <thread>
#include <chrono>
#include "../include/ControlEngine.hpp"
#include "../include/SimpleDevice.hpp"
#include "../include/Logger.hpp"

int main() {
    Logger::info("engine_demo: starting");
    ControlEngine engine;
    engine.initialize();

    SimpleDevice dev("fan-1", "Test Fan");
    dev.set_setpoint(1.0);
    engine.register_device(&dev);

    for (int i=0;i<20;i++) {
        engine.update();
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }

    engine.shutdown();
    Logger::info("engine_demo: finished");
    return 0;
}
