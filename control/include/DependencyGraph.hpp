#ifndef DEPENDENCY_GRAPH_HPP
#define DEPENDENCY_GRAPH_HPP

#include <unordered_map>
#include <vector>
#include <string>

class DependencyGraph {
public:
    DependencyGraph();
    void add_dependency(const std::string &a, const std::string &b);
    std::vector<std::string> topo_sort() const;
    bool has_cycle() const;
private:
    std::unordered_map<std::string, std::vector<std::string>> adj;
};

#endif
