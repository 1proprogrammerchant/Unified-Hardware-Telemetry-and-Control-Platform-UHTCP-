require 'thread'
require 'logger'

class AutomationScheduler
  def initialize
    @threads = []
    @mutex = Mutex.new
    @logger = Logger.new($stdout)
    @logger.progname = 'AutomationScheduler'
    @running = true
  end

  # Schedule a block to run in a new thread
  def schedule(&blk)
    raise ArgumentError, 'no block given' unless block_given?
    th = Thread.new do
      begin
        blk.call
      rescue => e
        @logger.error("scheduled job error: #{e.class}: #{e.message}\n#{e.backtrace.first}")
      end
    end
    @mutex.synchronize { @threads << th }
    th
  end

  # Schedule a recurring job
  def schedule_every(interval, &blk)
    schedule do
      loop do
        break unless @running
        begin
          blk.call
        rescue => e
          @logger.error("recurring job error: #{e.class}: #{e.message}")
        end
        sleep interval
      end
    end
  end

  def shutdown
    @running = false
    @mutex.synchronize { @threads.each(&:kill); @threads.clear }
  end

  def join
    @mutex.synchronize { @threads.each(&:join) }
  end
end
