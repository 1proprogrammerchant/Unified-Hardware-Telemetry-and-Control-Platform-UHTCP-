#!/usr/bin/env ruby

def cpu_temp
  begin
    raw = File.read('/sys/class/thermal/thermal_zone0/temp').to_i
    return raw / 1000.0
  rescue
    return nil
  end
end

loop do
  t = cpu_temp
  if t && t > 70
    puts "CPU hot (#{t} C) — consider turning on fan"
  else
    puts "CPU OK: #{t || 'n/a'}"
  end
  sleep 5
end
