#include "../include/ThreadPool.hpp"

ThreadPool::ThreadPool(size_t workers_) {
    size_t n = workers_==0?1:workers_;
    stop = false;
    for (size_t i=0;i<n;i++) {
        workers.emplace_back([this]{
            for(;;) {
                std::function<void()> task;
                {
                    std::unique_lock<std::mutex> lk(mtx);
                    cv.wait(lk, [this]{ return stop || !tasks.empty(); });
                    if (stop && tasks.empty()) return;
                    task = std::move(tasks.front()); tasks.pop();
                }
                task();
            }
        });
    }
}

ThreadPool::~ThreadPool() {
    {
        std::lock_guard<std::mutex> g(mtx);
        stop = true;
    }
    cv.notify_all();
    for (auto &w : workers) if (w.joinable()) w.join();
}

void ThreadPool::enqueue(std::function<void()> fn) {
    {
        std::lock_guard<std::mutex> g(mtx);
        tasks.push(std::move(fn));
    }
    cv.notify_one();
}
