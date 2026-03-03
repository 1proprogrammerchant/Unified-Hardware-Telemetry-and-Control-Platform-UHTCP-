require 'thread'

class AutomationScheduler
  def initialize; @threads = []; end
  def schedule(&blk) @threads << Thread.new(&blk); end
end
