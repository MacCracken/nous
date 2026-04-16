# Nous API Reference

> **Version**: 1.0.0 | **Language**: Cyrius 5.1.7
>
> Include via `include "src/nous.cyr"` — requires `alloc_init()` before use.

---

## Resolver

The main entry point. Wraps marketplace registry, system package DB, and optional recipe DB.

```cyrius
# Create a resolver
var r = payload(resolver_new(
    str_from("/var/lib/mela"),
    str_from("/var/cache/nous")));

# Set resolution strategy
r = resolver_with_strategy(r, strategy_system_first());

# Get current strategy
var s = resolver_strategy(r);  # returns Strategy ptr

# Attach recipe database
var rdb = recipe_db_load(str_from("/home/user/zugot"));
r = resolver_with_recipes(r, rdb);

# Enable resolution trace
r = resolver_with_trace(r);
```

### Resolution

```cyrius
# Resolve a single package
var res = resolver_resolve(r, str_from("curl"));
if (is_ok(res) == 1) {
    var pkg = payload(res);
    if (pkg != 0) {
        str_println(rp_name(pkg));   # "curl"
        str_println(rp_ver(pkg));    # "8.19.0"
        fmt_int(rp_src(pkg));        # SOURCE_SYSTEM
        fmt_int(rp_trust(pkg));      # 1
    }
}

# Resolve transitively (full dependency graph)
var names = vec_new();
vec_push(names, str_from("openssl"));
var plan_result = resolver_resolve_all(r, names);
if (is_ok(plan_result) == 1) {
    var plan = payload(plan_result);
    var pkgs = pl_packages(plan);    # vec of ResolvedPkg, install order
    var order = pl_order(plan);      # vec of Str names, install order
}

# Resolve with recipe fallback
var plan2 = resolver_resolve_all_with_recipes(r, names);
```

### Search, List, Updates

```cyrius
# Search across all sources
var sr = payload(resolver_search(r, str_from("audio")));
var results = sr_results(sr);        # vec of ResolvedPkg
var total = sr_total(sr);            # count

# List installed packages (sorted by name)
var list = payload(resolver_list(r));  # vec of InstalledPkg

# Check for system updates
var updates = payload(resolver_updates(r));  # vec of AvailUpdate

# Source detection
detect_source(str_from("acme/scanner"));    # SOURCE_MARKETPLACE
detect_source(str_from("com.example.app")); # SOURCE_FLUTTER_APP
detect_source(str_from("nginx"));           # SOURCE_UNKNOWN

# Check package source
resolver_is_mkt(r, str_from("myapp"));   # 1 or 0
resolver_is_sys(r, str_from("curl"));    # 1 or 0
```

### Resolution Trace

```cyrius
r = resolver_with_trace(r);
resolver_resolve(r, str_from("curl"));
var trace = resolver_get_trace(r);  # vec of Str
# e.g., ["marketplace: curl ... miss", "system: curl ... hit"]
resolver_clear_trace(r);
```

---

## Strategy

Controls which sources are checked and in what order.

```cyrius
strategy_default()                    # MarketplaceFirst (default)
strategy_system_first()               # SystemFirst
strategy_only(SOURCE_MARKETPLACE)     # OnlySource(Marketplace)
strategy_only(SOURCE_SYSTEM)          # OnlySource(System)
strategy_only(SOURCE_FLUTTER_APP)     # OnlySource(FlutterApp)
strategy_search_all()                 # SearchAll

# Compare strategies
strategy_eq(strategy_default(), strategy_default());  # 1
strategy_eq(strategy_default(), strategy_system_first());  # 0

# Access fields
st_kind(s);     # STRAT_MARKETPLACE_FIRST, etc.
st_only(s);     # PackageSource (when kind == STRAT_ONLY_SOURCE)
```

---

## Registry (Marketplace)

Local filesystem registry for marketplace packages.

```cyrius
var reg = payload(registry_new(str_from("/var/lib/mela")));

# Search by name or description
var results = registry_search(reg, str_from("scanner"));

# Get specific package
var pkg = registry_get(reg, str_from("my-agent"));
if (pkg != 0) {
    str_println(mpkg_name(pkg));     # "my-agent"
    str_println(mpkg_version(pkg));  # "1.0.0"
    fmt_int(mp_size(pkg));           # installed size in bytes
}

# List all installed
var list = registry_list(reg);  # vec of InstalledMPkg

# Refresh cache after changes
registry_reload(reg);

# Stub for future mela integration
registry_install_package(reg, str_from("/tmp/pkg.tar"), 0);
```

---

## System Package DB

Wrapper around apt-cache and dpkg-query. All commands use array-based exec (no shell injection).

```cyrius
var db = sysdb_new();
if (sysdb_available(db) == 1) {
    # Search apt
    var results = payload(sysdb_search(db, str_from("libssl")));

    # Check if installed
    var installed = payload(sysdb_is_installed(db, str_from("curl")));

    # Get installed package info
    var pkg = payload(sysdb_get_installed(db, str_from("curl")));

    # Detailed package info
    var info = payload(sysdb_info(db, str_from("curl")));

    # List all installed system packages
    var all = payload(sysdb_list(db));

    # Check for upgradable packages
    var updates = payload(sysdb_updates(db));
}
```

---

## Zugot Recipes

Parse `.cyml` recipe files from the zugot repository.

```cyrius
# Parse a single recipe
var recipe = recipe_parse_file(
    str_from("/home/user/zugot/base/curl.cyml"));
str_println(rc_name(recipe));      # "curl"
str_println(rc_ver(recipe));       # "8.19.0"
str_println(rc_license(recipe));   # "MIT"
var rt = rc_rt_deps(recipe);       # vec of Str: ["glibc", "openssl", ...]
var bd = rc_bd_deps(recipe);       # vec of Str: ["gcc", "make", ...]
str_println(rc_url(recipe));       # source tarball URL
str_println(rc_sha256(recipe));    # checksum

# Load entire recipe database (scans 11 categories)
var rdb = recipe_db_load(str_from("/home/user/zugot"));
fmt_int(recipe_db_count(rdb));     # e.g., 428

# Lookup by name
var curl = recipe_db_get(rdb, str_from("curl"));

# Get build order from build-order.txt
var order = recipe_db_order(rdb);  # vec of "category/name" Str

# Convert recipe to ResolvedPkg for graph integration
var pkg = recipe_to_resolved(curl);
```

---

## Version Constraints

SemVer parsing and constraint matching.

```cyrius
# Parse versions
var v = semver_parse(str_from("1.2.3"));
sv_major(v);  # 1
sv_minor(v);  # 2
sv_patch(v);  # 3

# Compare
semver_cmp(semver_parse(str_from("1.0")),
           semver_parse(str_from("2.0")));  # -1
semver_eq(v, v);  # 1

# Display
semver_to_str(v);  # Str "1.2.3"

# Parse constraints
var c = constraint_parse(str_from(">=1.2.0"));
ct_op(c);   # OP_GTE
ct_ver(c);  # SemVer ptr

# Supported operators: =, >=, >, <=, <, ^ (caret), ~ (tilde), *
constraint_parse(str_from("^1.0"));   # >=1.0.0, <2.0.0
constraint_parse(str_from("~1.2"));   # >=1.2.0, <1.3.0
constraint_parse(str_from("*"));      # any version

# Check if version satisfies constraint
constraint_matches(c, semver_parse(str_from("1.5.0")));  # 1
constraint_matches(c, semver_parse(str_from("0.9.0")));  # 0

# Check if two constraints can be simultaneously satisfied
constraints_compatible(
    constraint_parse(str_from(">=1.0")),
    constraint_parse(str_from("<2.0")));  # 1
```

---

## Dependency Graph

Build, analyze, and sort dependency graphs.

```cyrius
var g = dep_graph_new();

# Add packages with their deps
var deps_a = vec_new();
vec_push(deps_a, str_from("B"));
dep_graph_add(g, resolved_pkg_new(
    str_from("A"), str_from("1.0"), SOURCE_MARKETPLACE,
    str_from(""), 0, deps_a, 1));
dep_graph_add(g, resolved_pkg_new(
    str_from("B"), str_from("1.0"), SOURCE_MARKETPLACE,
    str_from(""), 0, vec_new(), 1));

# Check for cycles (returns 0 if none, vec of names if cycle)
var cycle = dep_graph_detect_cycle(g);

# Topological sort (Kahn's algorithm)
var order = dep_graph_topo_sort(g);  # vec of Str
```

---

## Error Handling

All fallible functions return `Ok(value)` or `Err(NousError)`.

```cyrius
var result = resolver_resolve(r, str_from(""));
if (is_err_result(result) == 1) {
    var err = payload(result);
    var kind = ne_kind(err);     # ERR_INVALID_PACKAGE_NAME
    var msg = ne_msg(err);       # the package name
    var detail = ne_detail(err); # "package name cannot be empty"

    # Human-readable message
    str_println(nous_error_msg(err));
    # "invalid package name ``: package name cannot be empty"
}

# Error kinds: ERR_COMMAND_EXEC, ERR_REGISTRY_IO,
# ERR_INVALID_MANIFEST, ERR_INVALID_PACKAGE_NAME,
# ERR_INVALID_VERSION_CONSTRAINT, ERR_DEPENDENCY_CYCLE,
# ERR_VERSION_CONFLICT

# Validate names before use
var vr = validate_package_name(str_from("valid-pkg"));  # Ok(1)
var vr2 = validate_package_name(str_from("bad;pkg"));   # Err(...)
```

---

## Typo Suggestions

```cyrius
var known = collect_known_names(r);  # vec of Str from all sources
var suggestion = suggest_package(str_from("ngins"), known);
if (suggestion != 0) {
    str_println(build_suggestion(str_from("ngins"), suggestion));
    # "package 'ngins' not found. Did you mean 'nginx'?"
}
```

---

## JSON Serialization

All public types have `to_json` / `from_json` functions.

```cyrius
# ResolvedPkg
var json = resolved_to_json(pkg);
var back = resolved_from_json(json);

# InstalledPkg
var json2 = installed_to_json(ipkg);
var back2 = installed_from_json(json2);

# AvailUpdate
var json3 = update_to_json(upd);

# Strategy
var json4 = strategy_to_json(strat);
var back4 = strategy_from_json(json4);

# SearchResult
var json5 = search_result_to_json(sr);

# NousErrorKind
var json6 = error_kind_to_json(err);
var back6 = error_kind_from_json(json6);

# Manifest
var json7 = manifest_to_json(m);

# PackageSource
source_to_str(SOURCE_SYSTEM);        # Str "system"
source_from_json(str_from("system")); # SOURCE_SYSTEM
```

---

## Package Sources

```cyrius
# Enum values
SOURCE_SYSTEM        # 0 — apt/dpkg
SOURCE_MARKETPLACE   # 1 — mela marketplace
SOURCE_FLUTTER_APP   # 2 — Flutter desktop apps
SOURCE_COMMUNITY     # 3 — community recipes
SOURCE_UNKNOWN       # 4 — not found

# Display
source_to_str(SOURCE_FLUTTER_APP);  # Str "flutter-app"
```

---

## Struct Accessors

All structs use `alloc`+`store64` construction with `load64` accessor functions.

| Struct | Constructor | Key Accessors |
|--------|-------------|---------------|
| ResolvedPkg | `resolved_pkg_new(name,ver,src,desc,size,deps,trusted)` | `rp_name`, `rp_ver`, `rp_src`, `rp_desc`, `rp_size`, `rp_deps`, `rp_trust` |
| InstalledPkg | `installed_pkg_new(name,ver,src,date,size,auto)` | `ip_name`, `ip_ver`, `ip_src`, `ip_date`, `ip_size`, `ip_auto` |
| InstalledMPkg | `installed_mpkg_new(manifest,size,path,at)` | `mpkg_name`, `mpkg_version`, `mpkg_desc`, `mpkg_runtime`, `mpkg_deps` |
| Strategy | `strategy_new(kind,only)` | `st_kind`, `st_only` |
| NousError | `nous_error_new(kind,msg,detail)` | `ne_kind`, `ne_msg`, `ne_detail` |
| SemVer | `semver_new(major,minor,patch)` | `sv_major`, `sv_minor`, `sv_patch` |
| Constraint | `constraint_new(op,ver)` | `ct_op`, `ct_ver` |
| SearchResult | `search_result_new(results,sources,total)` | `sr_results`, `sr_sources`, `sr_total` |
| ResPlan | `res_plan_new(packages,order)` | `pl_packages`, `pl_order` |
| Recipe | `recipe_alloc()` + `recipe_set_pkg/meta/deps` | `rc_name`, `rc_ver`, `rc_license`, `rc_rt_deps`, `rc_bd_deps`, ... |
