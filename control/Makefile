CC = g++
CFLAGS = -Iinclude -std=c++17 -O2
SRCS = src/Device.cpp src/Sensor.cpp src/Actuator.cpp src/DeviceRegistry.cpp \
	src/ControlEngine.cpp src/PolicyEngine.cpp src/CommandQueue.cpp src/PIDController.cpp \
	src/DependencyGraph.cpp src/SafetyManager.cpp src/Logger.cpp src/ThreadPool.cpp
SRCS += src/ThresholdPolicy.cpp src/SimpleDevice.cpp
OBJS = $(SRCS:.cpp=.o)

.PHONY: all clean

all: libcontrol.a

libcontrol.a: $(OBJS)
	ar rcs $@ $^

demo: libcontrol.a examples/main.o src/SimpleDevice.o
	$(CC) -Iinclude -std=c++17 -O2 -o engine_demo examples/main.o src/SimpleDevice.o libcontrol.a

examples/main.o: examples/main.cpp
	$(CC) -Iinclude -std=c++17 -O2 -c -o examples/main.o examples/main.cpp

src/SimpleDevice.o: src/SimpleDevice.cpp
	$(CC) -Iinclude -std=c++17 -O2 -c -o src/SimpleDevice.o src/SimpleDevice.cpp

clean:
	rm -f $(OBJS) libcontrol.a
