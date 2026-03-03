module Hardware
  def self.cpu_temperature
    # proxy to Go->Rust state in production
    0.0
  end

  def self.write_gpio(pin, value)
    # send command to server
  end
end
