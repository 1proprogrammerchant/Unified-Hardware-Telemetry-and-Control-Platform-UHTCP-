#!/usr/bin/env ruby
require 'logger'

LOGGER = Logger.new($stdout)
LOGGER.progname = 'automation:scripts'

def cpu_temp
  path = '/sys/class/thermal/thermal_zone0/temp'
  return nil unless File.exist?(path)

  raw = File.read(path).to_i
  (raw / 1000.0)
rescue Errno::EACCES => e
  LOGGER.warn("cannot read cpu temp: #{e.message}")
  nil
rescue => e
  LOGGER.error("unexpected error reading cpu temp: #{e.class}: #{e.message}")
  nil
end

def monitor_cpu(threshold: ENV.fetch('CPU_TEMP_THRESHOLD', 70).to_f, interval: ENV.fetch('CPU_TEMP_INTERVAL', 5).to_i)
  loop do
    t = cpu_temp
    if t && t > threshold
      LOGGER.warn("CPU hot (#{t.round(1)}°C) — consider turning on fan")
    else
      LOGGER.info("CPU OK: #{t ? t.round(1) : 'n/a'}°C")
    end
    sleep interval
  end
end

if __FILE__ == $PROGRAM_NAME
  monitor_cpu
end
