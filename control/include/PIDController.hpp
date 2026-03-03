#ifndef PID_CONTROLLER_HPP
#define PID_CONTROLLER_HPP

class PIDController {
public:
    PIDController(double kp = 1.0, double ki = 0.0, double kd = 0.0);
    double compute(double setpoint, double current, double dt_seconds);
    void reset();
private:
    double kp, ki, kd;
    double integral;
    double previous_error;
};

#endif
