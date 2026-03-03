#include "../include/PIDController.hpp"

PIDController::PIDController(double kp_, double ki_, double kd_) : kp(kp_), ki(ki_), kd(kd_), integral(0.0), previous_error(0.0) {}

double PIDController::compute(double setpoint, double current, double dt_seconds) {
    double error = setpoint - current;
    integral += error * dt_seconds;
    double derivative = 0.0;
    if (dt_seconds > 0.0) derivative = (error - previous_error) / dt_seconds;
    previous_error = error;
    return kp * error + ki * integral + kd * derivative;
}

void PIDController::reset() {
    integral = 0.0;
    previous_error = 0.0;
}
