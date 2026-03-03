#include "../include/DependencyGraph.hpp"
#include <stack>
#include <functional>

DependencyGraph::DependencyGraph() {}

void DependencyGraph::add_dependency(const std::string &a, const std::string &b) {
    adj[a].push_back(b);
}

bool DependencyGraph::has_cycle() const {
    enum State { UNVIS=0, VISITING=1, VISITED=2 };
    std::unordered_map<std::string, int> state;

    std::function<bool(const std::string&)> dfs = [&](const std::string &u) -> bool {
        state[u] = VISITING;
        auto it = adj.find(u);
        if (it != adj.end()) {
            for (auto &v : it->second) {
                if (state[v] == VISITING) return true;
                if (state[v] == UNVIS && dfs(v)) return true;
            }
        }
        state[u] = VISITED;
        return false;
    };

    for (auto &p : adj) {
        if (state[p.first] == UNVIS) {
            if (dfs(p.first)) return true;
        }
    }
    return false;
}

std::vector<std::string> DependencyGraph::topo_sort() const {
    std::vector<std::string> out;
    std::unordered_map<std::string, int> indeg;
    for (auto &p : adj) {
        if (indeg.find(p.first) == indeg.end()) indeg[p.first]=0;
        for (auto &v : p.second) indeg[v]++;
    }
    std::vector<std::string> q;
    for (auto &p : indeg) if (p.second==0) q.push_back(p.first);
    size_t idx=0;
    while (idx<q.size()) {
        auto u = q[idx++];
        out.push_back(u);
        auto it = adj.find(u);
        if (it!=adj.end()) {
            for (auto &v : it->second) {
                if (--indeg[v]==0) q.push_back(v);
            }
        }
    }
    return out;
}
