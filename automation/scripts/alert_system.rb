# Simple alert system script
# This script expects a `Hardware` API (provided by the sandbox) that responds
# to `cpu_temperature`. It will log when temperature crosses a threshold.
threshold = ENV.fetch('ALERT_CPU_THRESHOLD', 90).to_f
begin
  temp = Hardware.respond_to?(:cpu_temperature) ? Hardware.cpu_temperature : nil
  if temp && temp > threshold
    # Placeholder for alerting integration: send email, push, etc.
    puts({ type: 'alert', severity: 'critical', message: "CPU temperature #{temp}°C" }.to_json)
  end
rescue => e
  STDERR.puts "alert_system error: #{e.class}: #{e.message}"
end
