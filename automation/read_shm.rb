#!/usr/bin/env ruby
# Simple SHM inspector for the UHTCP shared-memory layout.
# Usage: ruby automation/read_shm.rb /path/to/shm_file

def read_be_u64(s)
  s.unpack1('Q>')
end

def read_be_u32(s)
  s.unpack1('L>')
end

path = ARGV[0] || '/tmp/uhtcp_state.bin'
unless File.exist?(path)
  STDERR.puts "file not found: #{path}"
  exit 2
end

f = File.open(path, 'rb')
hdr = f.read(12)
if !hdr || hdr.bytesize < 12
  STDERR.puts "file too small"
  exit 2
end
ver = read_be_u64(hdr[0,8])
len = read_be_u32(hdr[8,4])
payload = f.read(len) || ''
puts "version=#{ver} len=#{len}"
puts payload.force_encoding('UTF-8')
f.close
