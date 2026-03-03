#ifndef DEVICE_HPP
#define DEVICE_HPP

class Device {
public:
    virtual ~Device() {}
    virtual void read() = 0;
    virtual void write(int value) = 0;
};

#endif
