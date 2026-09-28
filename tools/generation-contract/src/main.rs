use serde_json::Value;
use std::{collections::BTreeSet, fs, path::PathBuf};

const EXPECTED_CONSUMER: &str = "pony-expres/poex-desktop-infra";
const EXPECTED_AUTHORITY_REPOSITORY: &str = "ORESoftware/ores-common-desktop-infra";
const EXPECTED_AUTHORITY_PR: u64 = 9;
const EXPECTED_AUTHORITY_REVISION: &str = "3fda402f724fc7e846d962645747cfd9d0cc21fc";
const EXPECTED_PRODUCT_ROLE: &str = "pony_actor_compute_uses_generation_handoff";

fn main() {
    if let Err(error) = run() {
        eprintln!("generation contract validation failed: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let path = PathBuf::from("ores-generation-contract.json");
    let bytes = fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let contract: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is invalid JSON: {error}", path.display()))?;

    require_string(&contract, "/schema", "ores.desktop-generation-consumer/v1")?;
    require_string(&contract, "/consumer/repository", EXPECTED_CONSUMER)?;
    require_string(&contract, "/consumer/role", "desktop_infra")?;
    require_string(
        &contract,
        "/authority/repository",
        EXPECTED_AUTHORITY_REPOSITORY,
    )?;
    require_u64(&contract, "/authority/pull_request", EXPECTED_AUTHORITY_PR)?;
    require_string(
        &contract,
        "/authority/revision",
        EXPECTED_AUTHORITY_REVISION,
    )?;

    let expected_lifecycle = [
        "prepare",
        "validate",
        "compile_build_generation",
        "stage",
        "health_check",
        "atomic_activate",
        "bounded_drain",
        "commit",
    ];
    let lifecycle = string_array(&contract, "/lifecycle")?;
    if lifecycle != expected_lifecycle {
        return Err("generation lifecycle order drifted".to_owned());
    }

    require_bool(&contract, "/rollback/required_before_commit", true)?;
    require_bool(&contract, "/rollback/retain_previous_generation", true)?;
    require_string(
        &contract,
        "/request_semantics/new_requests",
        "active_generation",
    )?;
    require_string(
        &contract,
        "/request_semantics/existing_requests",
        "pinned_generation",
    )?;
    require_bool(
        &contract,
        "/request_semantics/generation_identity_required",
        true,
    )?;
    require_string(
        &contract,
        "/routing/dynamic_route_authority",
        "shared_router_generation",
    )?;
    require_bool(&contract, "/routing/edge_proxy_route_authority", false)?;

    let stable_edges = string_array(&contract, "/routing/stable_edges")?
        .into_iter()
        .collect::<BTreeSet<_>>();
    for edge in ["nginx", "haproxy", "caddy"] {
        if !stable_edges.contains(edge) {
            return Err(format!("stable edge set is missing {edge}"));
        }
    }

    require_bool(
        &contract,
        "/middleware/beam_code_reload_requires_drain_or_otp_proof",
        true,
    )?;
    let roles = string_array(&contract, "/role_requirements")?
        .into_iter()
        .collect::<BTreeSet<_>>();
    for required in [
        "build_stage_activate",
        "health_before_activate",
        "stable_ingress_only",
        "no_dynamic_routes_in_edge_proxy",
        EXPECTED_PRODUCT_ROLE,
    ] {
        if !roles.contains(required) {
            return Err(format!("role_requirements is missing {required}"));
        }
    }

    require_bool(&contract, "/verification/shared_conformance_required", true)?;
    require_bool(&contract, "/verification/product_e2e_required", true)?;
    require_string(&contract, "/verification/promotion_state", "candidate")?;

    println!("generation contract OK: {EXPECTED_CONSUMER}");
    return Ok(());
}

fn string_array<'a>(value: &'a Value, pointer: &str) -> Result<Vec<&'a str>, String> {
    let array = value
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{pointer} must be an array"))?;
    return array
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| format!("{pointer} entries must be strings"))
        })
        .collect();
}

fn require_string(value: &Value, pointer: &str, expected: &str) -> Result<(), String> {
    if value.pointer(pointer).and_then(Value::as_str) != Some(expected) {
        return Err(format!("{pointer} must equal {expected:?}"));
    }
    return Ok(());
}

fn require_bool(value: &Value, pointer: &str, expected: bool) -> Result<(), String> {
    if value.pointer(pointer).and_then(Value::as_bool) != Some(expected) {
        return Err(format!("{pointer} must equal {expected}"));
    }
    return Ok(());
}

fn require_u64(value: &Value, pointer: &str, expected: u64) -> Result<(), String> {
    if value.pointer(pointer).and_then(Value::as_u64) != Some(expected) {
        return Err(format!("{pointer} must equal {expected}"));
    }
    return Ok(());
}
