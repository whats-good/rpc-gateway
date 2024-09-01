- TODO: consider using CompactString instead of regular strings for configs
- TODO: in broadcast channels, use Arc<str> instead of Arc<String> to avoid unnecessary cloning
  (Broadcasting a payload will `Clone` it each and every single time.)
- TODO: look into DashSets instead of HashSets for performant concurrent hash sets. keys are shared into lock groups, so if 2 unrelated locks come into the same hashset, they won't block for no reason.
- TODO: System allocator -> jemalloc (jemalloc is suppose to be faster for multithreaded programs because it uses per-thread arenas which reduces contention for memory allocation. That sounds good to me, so let's change our allocator to jemalloc.) (https://arc.net/l/quote/yxriimmz)
