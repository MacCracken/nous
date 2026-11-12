# Nous API Reference

> **Version**: 1.7.1 | **Language**: Cyrius 6.7.18 (pinned in `cyrius.cyml`)
>
> **In-tree:** `include "src/nous.cyr"` (the barrel). **As a dependency:** pull
> `dist/nous.cyr` (see [ADR 0002](adr/0002-flat-modules-dist-bundle.md)), and
> also supply bayan's `bayan_toml_*` and `bayan_json_*` functions: either the
> `dist/bayan-toml.cyr` + `dist/bayan-json.cyr` profiles or the full stdlib
> `bayan` fold, not both ([ADR 0004](adr/0004-bayan-toml-profile-dependency.md)).
> Call `alloc_init()` before anything else.
>
> **Results are value-form** (cyrius 6.6.0+). A fallible function returns a
> `(tag, payload)` pair, so bind both halves: `var t, v = f();`, then test the
> tag with `is_ok(t)` / `is_err_result(t)`. To propagate, re-emit both halves:
> `if (is_err_result(t) == 1) { return Err(v); }`. A bare `return v;` hands the
> caller the payload as its tag, so an Err reads as Ok (CHANGELOG 1.3.2).
>
> **Which functions return a Result (1.6.0):** every one declared `: Result` in
> `src/`. That is 26 public functions: `nous_err`, `err_*`,
> `validate_package_name`, `nous_capture*`, `nous_cmd_capture`, `registry_new`,
> `registry_install_package`, the seven `sysdb_*` queries (not `sysdb_new` or
> `sysdb_available`), `resolve_sys`, `resolver_new`,
> `resolver_resolve`, `resolver_search`, `resolver_list`, `resolver_updates`,
> `resolver_resolve_all*`, `finalize_graph` and `resolver_check_updates`.
> Everything else returns a plain `i64`. The compiler refuses a one-variable
> bind of a Result (`var r = f();`) whether or not it is annotated.
>
> **Per-function reference:** `cyrius doc src/<module>.cyr` prints a one-line
> summary for every public function. Each line is the comment directly above
> the function, and CI fails if any public function lacks one (1.6.0).

---

## Resolver

The main entry point. It wraps the marketplace registry, the system package DB
and an optional recipe DB.

```cyrius
# Create a resolver (Result: registry I/O can fail)
var r_t, r = resolver_new(str_from("/var/lib/mela"), str_from("/var/cache/nous"));
if (is_err_result(r_t) == 1) { str_println(nous_error_msg(r)); }

# Set the resolution strategy (returns r)
resolver_with_strategy(r, strategy_system_first());
var s = rs_strat(r);                 # the current Strategy

# Attach a recipe database (returns r)
var rdb = recipe_db_load(str_from("/home/user/zugot"));
resolver_with_recipes(r, rdb);

# Turn on the resolution trace (returns r)
resolver_with_trace(r);
```

### Resolution

```cyrius
# Resolve a single package: Ok(ResolvedPkg), Ok(0) when not found, or Err
var res_t, pkg = resolver_resolve(r, str_from("curl"));
if (is_ok(res_t) == 1 && pkg != 0) {
    str_println(rp_name(pkg));       # "curl"
    str_println(rp_ver(pkg));        # "8.19.0"
    fmt_int(rp_src(pkg));            # SOURCE_SYSTEM
    fmt_int(rp_trust(pkg));          # 1 for apt; 0 for marketplace / community until
                                     # sigil verifies signatures (1.5.3, roadmap 1.9.x)
}

# Resolve transitively (full dependency graph, install order)
var names = vec_new();
vec_push(names, str_from("openssl"));
var plan_t, plan = resolver_resolve_all(r, names);
if (is_ok(plan_t) == 1) {
    var pkgs = pl_packages(plan);    # vec of ResolvedPkg, install order (deps first)
    var order = pl_order(plan);      # the same order, as Str names (1.5.3: was reversed)
    var gaps = nous_pl_unresolved(plan);   # apt names apt could not show (virtual
                                           # packages, alternatives): nous_unres_name /
                                           # nous_unres_requirer. Listed, not fatal.
}
# Errors (1.5.3), never a silently partial plan:
#   NOUS_ERR_NOT_FOUND           a root, or a dependency of a non-apt package, that
#                                no source has (detail: who needed it + a suggestion)
#   NOUS_ERR_DEPENDENCY_CYCLE    a cycle touching a non-apt package (apt-only cycles
#                                are accepted, as Debian allows; ADR 0005)
#   NOUS_ERR_VERSION_CONFLICT    the constraints on a dependency have no common version,
#                                or the version available satisfies one of them not
#   NOUS_ERR_INVALID_VERSION_CONSTRAINT  a manifest constraint nous does not support
# A dependency of an apt package resolves from apt only, whatever the strategy, so
# a marketplace package cannot be spliced into apt's closure (1.5.3).

# Same, with zugot recipes as a fallback source
var plan2_t, plan2 = resolver_resolve_all_with_recipes(r, names);
```

### Search, List, Updates

```cyrius
# Search across all sources (registry entries win on a name clash)
var sr_t, sr = resolver_search(r, str_from("audio"));
var results = sr_results(sr);        # vec of ResolvedPkg
var total = sr_total(sr);            # count
var srcs = sr_sources(sr);           # [marketplace, flutter-app, system]

# List installed packages (sorted by name)
var ls_t, list = resolver_list(r);   # vec of InstalledPkg

# System (apt) updates only
var up_t, updates = resolver_updates(r);   # vec of AvailUpdate

# Source detection (heuristic, by name shape)
detect_source(str_from("acme/scanner"));    # SOURCE_MARKETPLACE
detect_source(str_from("com.example.app")); # SOURCE_FLUTTER_APP
detect_source(str_from("nginx"));           # SOURCE_UNKNOWN

# Is it an installed marketplace package?
resolver_is_mkt(r, str_from("myapp"));      # 1 or 0
```

A registry package whose manifest `runtime` is `"flutter"` is labelled
`SOURCE_FLUTTER_APP` by resolve, search and list alike; any other registry
package is `SOURCE_MARKETPLACE` (1.6.1; search and list used to say
marketplace for both).

Under `strategy_search_all()`, `resolver_resolve` asks every source, and a name
found in more than one is `Err(NOUS_ERR_AMBIGUOUS_SOURCE)` (1.5.3). Pick a
source with `strategy_only()`.

Since 1.5.1, `resolver_search`, `resolver_list` and `resolver_check_updates`
return `Err(NOUS_ERR_COMMAND_EXEC)` when the system leg's capture fails (see
[Child-output capture](#child-output-capture)). They no longer return Ok with
the system results silently missing.

### Multi-source update detection (1.3.0)

nous knows what is **installed** but not what is **available** outside apt. The
caller supplies that through a lookup callback, and nous does the comparison:
Debian order (deb-version(7)) for community versions, SemVer for marketplace
and Flutter (1.5.3).

```cyrius
# avail_fn(name, source) -> available-version Str, or 0 / "" when unknown
fn my_avail(name, source): i64 { return mela_latest(name); }   # caller-defined

var installed = ...;                 # vec of InstalledPkg (e.g. ark's PackageDb)
var cu_t, ups = resolver_check_updates(r, installed, &my_avail);
# ups: vec of AvailUpdate. The apt leg is swept globally; SOURCE_SYSTEM entries
# in `installed` are ignored, and every other entry is checked via avail_fn.
# Pass 0 as avail_fn for apt-only (the same as resolver_updates).

update_is_newer(str_from("1.4.0"), str_from("1.3.9"));   # 1
update_is_newer(str_from("1.4.0"), str_from("1.4.0"));   # 0 (equal or older)
update_is_newer(str_from("v2.0.0"), str_from("1.0.0"));  # 1 (a leading v is fine)
nous_update_is_newer_src(str_from("10.3p2"), str_from("10.3p1"), SOURCE_COMMUNITY);  # 1
nous_deb_version_cmp(str_from("1:1.0-1"), str_from("2.0-1"));  # > 0: the epoch wins
```

### Resolution Trace

```cyrius
resolver_with_trace(r);
var t, p = resolver_resolve(r, str_from("curl"));
var trace = resolver_get_trace(r);   # vec of Str
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
strategy_eq(strategy_default(), strategy_default());       # 1
strategy_eq(strategy_default(), strategy_system_first());  # 0

# Access fields
st_kind(s);     # STRAT_MARKETPLACE_FIRST, etc.
st_only(s);     # PackageSource (when kind == STRAT_ONLY_SOURCE)
```

---

## Registry (Marketplace)

A local filesystem registry for marketplace packages. A stub until mela
integration lands (roadmap 1.9.x). Each `installed/<dir>/manifest.json` is read
whole (up to `NOUS_FILE_MAX`, 1 MiB) and parsed by structure with bayan's JSON
parser: `agent.name`, `agent.version`, `agent.description`, `dependencies`,
`runtime` (1.5.3). Directories are scanned in byte order.

```cyrius
var reg_t, reg = registry_new(str_from("/var/lib/mela"));

# Search by name or description
var results = registry_search(reg, str_from("scanner"));

# Get a specific package (0 if absent)
var pkg = nous_registry_get(reg, str_from("my-agent"));
if (pkg != 0) {
    str_println(mpkg_name(pkg));     # "my-agent"
    str_println(mpkg_version(pkg));  # "1.0.0"
    fmt_int(mp_size(pkg));           # installed size in bytes
}

# List all installed
var list = nous_registry_list(reg);       # vec of InstalledMPkg

# Refresh after changes
registry_reload(reg);

# Stub for future mela integration: always Ok(0)
var ip_t, ip = registry_install_package(reg, str_from("/tmp/pkg.tar"), 0);
```

---

## System Package DB

A wrapper around `apt-cache`, `dpkg-query` and `apt-get -s`. Every query is
array-form exec (no shell) with an absolute tool path, and only names in Debian
Policy syntax (lowercase alphanumeric first, then `[a-z0-9+.-]`) ever reach an
argv: an option-shaped name such as `-c/tmp/x` is never passed (1.5.3). Parsing
is in pure `_sysdb_parse_*` functions (unit-tested and fuzzed without apt).

```cyrius
var db = sysdb_new();                # apt detected via nous_which_exists
if (sysdb_available(db) == 1) {
    var s_t, results = sysdb_search(db, str_from("libssl"));      # vec of ResolvedPkg
    var i_t, installed = sysdb_is_installed(db, str_from("curl")); # 1 or 0
    var g_t, pkg = sysdb_get_installed(db, str_from("curl"));      # InstalledPkg or 0
    var n_t, info = sysdb_info(db, str_from("curl"));              # ResolvedPkg or 0
    var l_t, all = sysdb_list(db);                                 # vec of InstalledPkg
    var u_t, ups = sysdb_updates(db);                              # vec of AvailUpdate
    var m_t, many = sysdb_info_many(db, names);   # map cstr -> ResolvedPkg, ONE apt call
}
```

1.5.3:
- **`sysdb_info`** reads the candidate only (`apt-cache show --no-all-versions`).
  `Pre-Depends` count as dependencies; `:any` / `:<arch>` qualifiers are stripped;
  the first `|` alternative is kept.
- **"Installed"** is dpkg's `${db:Status-Status}` = `installed`, so a held package
  is installed and a removed-with-config one is not.
- **`sysdb_updates`** reads `apt-get -s dist-upgrade` `Inst` lines, not `apt list`
  (which apt(8) says is not a script interface).
- **`sysdb_search`** escapes the query, which apt-cache treats as a regex.
- **Transitive resolution** fetches the apt side one breadth-first wave per
  `apt-cache` call (`sysdb_info_many`). apt spends ~430 ms per call loading its
  cache, so curl's closure went from 14.7 s to 2.9 s.

Every `sysdb_*` query captures the tool's **complete** output (1.5.1). A query
whose output does not fit, or that the idle deadline cuts short, returns
`Err(NOUS_ERR_COMMAND_EXEC)`, never a partial result. With apt absent, every
query is `Ok` with an empty or 0 value.

### Child-output capture

```cyrius
# argv is a vec of cstr; args[0] is an absolute path (the child gets an empty env).
var args = vec_new();
vec_push(args, "/usr/bin/dpkg-query");
vec_push(args, "-W");
var t, out = nous_capture(args);        # default limits: 64 KiB start, 16 MiB max
# Ok(Str): the complete output, an owned copy that later captures cannot change
# Ok(0):   no output (or the command could not start)
# Err(e):  NOUS_ERR_COMMAND_EXEC: over the limit, or cut by the idle deadline

var t2, out2 = nous_capture_n(args, 1024, 4096);   # explicit start/max bytes
var t3, out3 = nous_cmd_capture(str_from("echo hi"));     # via /bin/sh -c

nous_which_exists("apt-cache");          # 1 if on PATH, else 0
which_path("apt-cache");                 # absolute path (cstr) or 0
```

Limits: `NOUS_CAPTURE_INIT` (65,536) and `NOUS_CAPTURE_MAX` (16,777,216). Each
run reads up to `cap + 1` bytes. When the output does not fit, the command
re-runs with double the buffer, up to the max. nous only runs read-only queries
this way, so a re-run is safe. When the caller has set no deadline
(`proc_timeout_ms() == 0`), nous applies `NOUS_CAPTURE_IDLE_MS` (60 s) around its
own runs and then restores 0; set it to 0 to disable (1.5.3).

---

## Zugot Recipes

Parse `.cyml` recipe files from the zugot repository.

```cyrius
# Parse a single recipe (0 if unreadable or it has no [package] section)
var recipe = nous_recipe_parse_file(str_from("/home/user/zugot/base/curl.cyml"));
str_println(rc_name(recipe));      # "curl"
str_println(rc_ver(recipe));       # "8.19.0"
str_println(rc_license(recipe));   # "MIT"
var rt = rc_rt_deps(recipe);       # vec of Str: ["glibc", "openssl", ...]
var bd = rc_bd_deps(recipe);       # vec of Str: ["gcc", "make", ...]
str_println(rc_url(recipe));       # source URL; for a github_release recipe, its
                                   # .../releases page, not the asset
str_println(rc_sha256(recipe));    # checksum
# A release-asset source as written (1.7.0; "" when none). The asset URL is
# takumi's to resolve at fetch time, through the GitHub API; nous does no network.
str_println(nous_rc_github_release(recipe));   # "owner/repo"
str_println(nous_rc_release_asset(recipe));    # "name-*-linux-amd64.tar.gz"

# Load the whole recipe database: every category directory, in byte order (1.5.3).
# On a name collision the first wins (base first, then categories and files in
# byte order), and the rest are reported.
var rdb = recipe_db_load(str_from("/home/user/zugot"));
fmt_int(recipe_db_count(rdb));     # 535 for zugot today
var dups = nous_recipe_db_duplicates(rdb);   # vec: nous_rdup_name / _kept / _ignored

# Look up by name (0 if absent)
var curl = recipe_db_get(rdb, str_from("curl"));

# Build order from build-order.txt
var order = recipe_db_order(rdb);  # vec of "category/name" Str

# Convert a recipe to a ResolvedPkg for the graph
var pkg = recipe_to_resolved(curl);

# The CYML parser itself: a vec of bayan_toml sections
var sections = nous_cyml_parse(str_from("[package]\nname = \"x\"\n"));
var sec = cyml_section_by_name(sections, "package");
cyml_get(sec, "name");             # Str "x", or 0
```

---

## Version Constraints

SemVer parsing and constraint matching.

```cyrius
# Parse versions: "1", "1.2" and "1.2.3" forms, with an optional "-pre" and
# "+build" (1.7.0). Returns 0 only when a component overflows; a non-digit ends
# a component.
var v = semver_parse(str_from("1.2.3-rc.1+build.5"));
sv_major(v);        # 1
sv_minor(v);        # 2
sv_patch(v);        # 3
nous_sv_pre(v);     # Str "rc.1"     (0 for a release)
nous_sv_build(v);   # Str "build.5"  (0 when absent)

# Compare by SemVer 2.0.0 precedence (§11): a pre-release sorts before its
# release; build metadata is ignored (§10).
semver_cmp(semver_parse(str_from("1.0.0-rc.1")), semver_parse(str_from("1.0.0")));  # -1
semver_eq(semver_parse(str_from("1.0.0+a")), semver_parse(str_from("1.0.0+b")));    # 1
# pub_semver order, for Flutter: precedence, then build (none first).
nous_semver_cmp_pub(semver_parse(str_from("1.0.0+1")), semver_parse(str_from("1.0.0+2")));  # -1

# Display
semver_to_str(v);  # Str "1.2.3-rc.1+build.5"

# Parse constraints (a version range; docs/sources.md has the full table)
var c = constraint_parse(str_from(">=1.2.0"));
ct_op(c);   # OP_GTE
ct_ver(c);  # SemVer
nous_constraint_error(c);   # 0, or a Str saying why the text is not supported

# Operators: =, >=, >, <=, <, ^ (caret), ~ (tilde), *; a leading v is fine.
constraint_parse(str_from("^1.0"));   # >=1.0.0, <2.0.0
constraint_parse(str_from("~1.2"));   # >=1.2.0, <1.3.0
constraint_parse(str_from("^0"));     # >=0.0.0, <1.0.0   (partials per npm/Cargo)
constraint_parse(str_from("1.2"));    # >=1.2.0, <1.3.0   (bare partial = X-range)
constraint_parse(str_from("*"));      # any release version
constraint_parse(str_from(">=1.1.0-rc.1"));  # a pre-release bound (1.7.0, full M.m.p only)
# Compound ranges (1.7.1): npm and Cargo forms, in one grammar (ADR 0007).
constraint_parse(str_from(">=1.2 <2.0"));            # AND, npm (whitespace)
constraint_parse(str_from(">= 1.2, < 1.5"));         # AND, Cargo (commas)
constraint_parse(str_from("1.2.7 || >=1.2.9 <2"));   # OR of comparator sets
constraint_parse(str_from("1.2.3 - 2.3"));           # hyphen: >=1.2.3 <2.4.0
# One comparator stays a simple constraint; more is ct_op == OP_SET, with the
# comparator sets in nous_ct_sets(c): a vec (OR) of vecs (AND) of simple ones.
# Refused (1.5.3; they used to parse silently): "!=1.0", "==1.0", "~>1.2", ">>1",
# "latest", a pre-release on a partial ("1.2-rc"), a malformed pre-release
# ("1.0.0-01", "1.0.0-rc..1"), a single "|", a stray ",", an operator with no
# version, a hyphen range that is not "A - B". A refused constraint matches
# nothing. Build metadata in a constraint is accepted and ignored.

# Does a version satisfy a constraint?
constraint_matches(c, semver_parse(str_from("1.5.0")));  # 1
constraint_matches(c, semver_parse(str_from("0.9.0")));  # 0
# A pre-release matches only a constraint that names a pre-release of the same
# M.m.p (npm / Cargo; ADR 0006), so ">=1.0.0" and "*" take no pre-release:
constraint_matches(constraint_parse(str_from(">=1.0.0")), semver_parse(str_from("1.1.0-rc.1")));       # 0
constraint_matches(constraint_parse(str_from(">=1.1.0-rc.1")), semver_parse(str_from("1.1.0-rc.2")));  # 1

# Can two constraints be satisfied at once? A true range intersection (1.5.3).
constraints_compatible(constraint_parse(str_from(">=1.0")),
                       constraint_parse(str_from("<2.0")));  # 1
constraints_compatible(constraint_parse(str_from(">1.2.0")),
                       constraint_parse(str_from("<1.3.0")));  # 1 (was 0)
# 1.7.0: an overlap holding only pre-releases needs both sides to opt in.
constraints_compatible(constraint_parse(str_from(">1.0.0")),
                       constraint_parse(str_from("<1.0.1")));  # 0 (only 1.0.1-x lie between)

# Debian version order (deb-version(7)), for apt and community versions
nous_deb_version_cmp(str_from("2.30-0ubuntu2"), str_from("2.30~rc1-1"));  # > 0
```

---

## Dependency Graph

Build, analyze and sort dependency graphs.

```cyrius
var g = dep_graph_new();

# Add packages with their deps
var deps_a = vec_new();
vec_push(deps_a, str_from("B"));
dep_graph_add(g, resolved_pkg_new(str_from("A"), str_from("1.0"), SOURCE_MARKETPLACE,
    str_from(""), 0, deps_a, 1));
dep_graph_add(g, resolved_pkg_new(str_from("B"), str_from("1.0"), SOURCE_MARKETPLACE,
    str_from(""), 0, vec_new(), 1));

# Cycle check: 0 if none, else the names on a cycle [a, ..., a] (iterative Tarjan,
# 1.5.3; any cycle, whatever its sources; finalize_graph accepts apt-only ones)
var cycle = dep_graph_detect_cycle(g);

# Topological sort (Kahn's algorithm): dependents first; partial on a cycle
var order = dep_graph_topo_sort(g);  # vec of Str
```

---

## Error Handling

Every fallible function returns `Ok(value)` or `Err(NousError)` as a value-form
pair.

```cyrius
var t, e = resolver_resolve(r, str_from(""));
if (is_err_result(t) == 1) {
    var kind = ne_kind(e);       # NOUS_ERR_INVALID_PACKAGE_NAME
    var msg = ne_msg(e);         # the package name
    var detail = ne_detail(e);   # "package name cannot be empty"
    str_println(nous_error_msg(e));
    # "invalid package name ``: package name cannot be empty"
}

# Validate a name before use
var v_t, v = validate_package_name(str_from("valid-pkg"));   # Ok(1)
var w_t, w = validate_package_name(str_from("bad;pkg"));     # Err(...)
```

| Kind | Value | Raised by |
|---|---|---|
| `NOUS_ERR_COMMAND_EXEC` | 0 | a capture over its limit or cut by the deadline |
| `NOUS_ERR_REGISTRY_IO` | 1 | reserved (not raised today) |
| `NOUS_ERR_INVALID_MANIFEST` | 2 | reserved (not raised today; an unreadable manifest is skipped) |
| `NOUS_ERR_INVALID_PACKAGE_NAME` | 3 | `validate_package_name` (incl. a leading `-`, 1.5.3) |
| `NOUS_ERR_INVALID_VERSION_CONSTRAINT` | 4 | `resolver_resolve_all*`: a manifest constraint nous does not support (1.5.3) |
| `NOUS_ERR_DEPENDENCY_CYCLE` | 5 | `resolver_resolve_all*` |
| `NOUS_ERR_VERSION_CONFLICT` | 6 | `resolver_resolve_all*` |
| `NOUS_ERR_NOT_FOUND` | 7 | `resolver_resolve_all*`: a name no source has (1.5.3) |
| `NOUS_ERR_AMBIGUOUS_SOURCE` | 8 | `resolver_resolve` under SearchAll (1.5.3) |

The values are the JSON wire format (`{"kind":N}`). The names were bare `ERR_*`
before 1.5.1 (CHANGELOG 1.5.1 has the migration).

---

## Typo Suggestions

```cyrius
var known = collect_known_names(r);  # vec of Str from all sources
var suggestion = suggest_package(str_from("ngins"), known);   # Levenshtein
if (suggestion != 0) {
    str_println(build_suggestion(str_from("ngins"), suggestion));
    # "package 'ngins' not found. Did you mean 'nginx'?"
}
```

---

## JSON Serialization

Every public type has a `*_to_json` and a `*_from_json` that roundtrips every
field (1.5.3). Strings are escaped on output (`"`, `\`, control characters) and
decoded on input, by the stdlib escaper and bayan's JSON parser.

```cyrius
var json = resolved_to_json(pkg);          var back = resolved_from_json(json);
var json2 = installed_to_json(ipkg);       var back2 = installed_from_json(json2);
var json3 = update_to_json(upd);         var back3 = update_from_json(json3);
var json4 = strategy_to_json(strat);       var back4 = strategy_from_json(json4);
var json5 = search_result_to_json(sr);   var back5 = search_result_from_json(json5);
var json6 = error_kind_to_json(err);       var back6 = error_kind_from_json(json6);
var json7 = manifest_to_json(m);         var back7 = manifest_from_json(json7);
var json8 = agent_info_to_json(a);       var back8 = agent_info_from_json(json8);

# PackageSource
source_to_str(SOURCE_SYSTEM);              # Str "system"
source_from_json(str_from("system"));      # SOURCE_SYSTEM
```

---

## Package Sources

```cyrius
SOURCE_SYSTEM        # 0 — apt/dpkg
SOURCE_MARKETPLACE   # 1 — mela marketplace
SOURCE_FLUTTER_APP   # 2 — Flutter desktop apps
SOURCE_COMMUNITY     # 3 — community recipes
SOURCE_UNKNOWN       # 4 — not found

source_to_str(SOURCE_FLUTTER_APP);  # Str "flutter-app"
```

---

## Deprecated (removed in 2.0.0)

They still work and warn at the call site (`#deprecated`). Their bare names
collide with other AGNOS libraries in cyrius's flat namespace (none is live in
ark today).

| Deprecated | Use instead | Collides with |
|---|---|---|
| `cmd_run(cmd)` → Str or 0 | `nous_cmd_capture(cmd)` → Result | thoth's `cmd_run(line)` |
| `which_exists(name)` | `nous_which_exists(name)` | kavach's `which_exists(name)` |
| `to_cstr(s)` (1.5.3) | stdlib `str_cstr(s)` | hadara, hoosh, vidya |
| `join_path(dir, name)` (1.5.3) | stdlib `path_join(dir, name)` | commandress |
| `registry_get` / `registry_list` (1.5.3) | `nous_registry_get` / `nous_registry_list` | bote, hoosh, szal, thoth |
| `search_result_new` (1.5.3) | `nous_search_result_new` | daimon (5 args: a hard error if bundled together) |
| `recipe_parse_file` (1.5.3) | `nous_recipe_parse_file` | takumi |

---

## Struct Accessors

Every struct is built with `alloc` + `store64` and read through `load64`
accessor functions; fields sit at `index * 8`. `src/types.cyr` has the full
layouts.

| Struct | Constructor | Key Accessors |
|--------|-------------|---------------|
| ResolvedPkg | `resolved_pkg_new(name,ver,src,desc,size,deps,trusted)` | `rp_name`, `rp_ver`, `rp_src`, `rp_desc`, `rp_size`, `rp_deps`, `rp_trust` |
| InstalledPkg | `installed_pkg_new(name,ver,src,date,size,auto)` | `ip_name`, `ip_ver`, `ip_src`, `ip_date`, `ip_size`, `ip_auto` |
| InstalledMPkg | `installed_mpkg_new(manifest,size,path,at)` | `mpkg_name`, `mpkg_version`, `mpkg_desc`, `mpkg_runtime`, `mpkg_deps` |
| AvailUpdate | `avail_update_new(name,src,inst_ver,avail_ver,changelog)` | see `src/types.cyr` |
| Strategy | `strategy_new(kind,only)` | `st_kind`, `st_only` |
| NousError | `nous_error_new(kind,msg,detail)` | `ne_kind`, `ne_msg`, `ne_detail` |
| SemVer | `semver_new(major,minor,patch)` | `sv_major`, `sv_minor`, `sv_patch` |
| Constraint | `constraint_new(op,ver)` | `ct_op`, `ct_ver`, `ct_prec`, `nous_constraint_error` |
| SearchResult | `nous_search_result_new(results,sources,total)` | `sr_results`, `sr_sources`, `sr_total` |
| ResPlan | `res_plan_new(packages,order)` | `pl_packages`, `pl_order`, `nous_pl_unresolved` |
| Recipe | `recipe_alloc()` + `recipe_set_pkg/meta/deps` | `rc_name`, `rc_ver`, `rc_license`, `rc_rt_deps`, `rc_bd_deps`, `rc_url`, `rc_sha256`, ... |
