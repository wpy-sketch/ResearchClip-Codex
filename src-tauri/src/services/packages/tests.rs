use super::*;
fn trust() -> Trust {
    Trust {
        schema_version: 1,
        catalog_url: None,
        catalog_public_key: None,
        updater_url: None,
        updater_public_key: None,
    }
}
#[test]
fn v500_compiled_installs_rescope_only_exact_reviewed_metadata_and_require_real_recheck() {
    compiled_upgrade_case(include_str!("../../../resources/package-builtin-v500.json"));
}
#[test]
fn v501_compiled_installs_rescope_only_exact_reviewed_metadata_and_require_real_recheck() {
    compiled_upgrade_case(include_str!("../../../resources/package-builtin-v501.json"));
}
fn compiled_upgrade_case(legacy_payload: &str) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("packages");
    let service = PackageManager::production(root.clone()).unwrap();
    let old: Catalog = serde_json::from_str(legacy_payload).unwrap();
    let p = old
        .packages
        .iter()
        .find(|p| p.id == "model.ocr-line")
        .unwrap()
        .clone();
    let directory = format!("{}-{}-{}", p.id, p.version, &p.files[0].sha256[..12]);
    let record = Installed {
        package: p,
        catalog_proof: Some(Envelope {
            payload: legacy_payload.into(),
            signature: "BUILTIN_MANIFEST_V1".into(),
        }),
        directory: directory.clone(),
        enabled: true,
        readiness: "READY".into(),
        installed_at: 123,
    };
    let managed = root.join("installed").join(&directory);
    std::fs::create_dir(&managed).unwrap();
    std::fs::write(managed.join("preserve.txt"), b"owned migration marker").unwrap();
    for variant in 0..4 {
        let mut entry = record.clone();
        entry.catalog_proof.as_mut().unwrap().payload = legacy_payload.replace("\r\n", "\n");
        if variant == 1 {
            entry.catalog_proof.as_mut().unwrap().payload =
                legacy_payload.replace("\r\n", "\n").replace('\n', "\r\n");
        }
        if variant == 2 {
            entry.package.files[0].url = "https://github.com/evil/unreviewed".into();
        }
        if variant == 3 {
            entry.catalog_proof.as_mut().unwrap().payload.push(' ');
        }
        save(
            &root,
            "registry.json",
            &PackageRegistry {
                installed: vec![entry.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        let reopened = PackageManager::production(root.clone()).unwrap();
        let s = reopened.snapshot().unwrap();
        let actual = &s.registry.installed[0];
        if variant >= 2 {
            assert!(actual.package == entry.package);
            assert!(reopened.authenticated(actual).is_err());
        } else {
            assert_eq!(
                actual.package.app_version,
                format!("={}", env!("CARGO_PKG_VERSION"))
            );
            assert_eq!(actual.readiness, "RECHECK_REQUIRED");
            assert!(reopened.authenticated(actual).is_ok());
            assert_eq!(actual.installed_at, 123);
            assert!(actual.enabled);
        }
        assert_eq!(
            std::fs::read(managed.join("preserve.txt")).unwrap(),
            b"owned migration marker"
        );
    }
    drop(service);
}
#[test]
fn paths_and_code_payloads_are_rejected() {
    for s in ["../escape", "a/b", "a\\b", "a:ads", "CON", "a.", ".."] {
        assert!(!safe_name(s), "{s}");
    }
    for s in ["x.dll", "x.exe", "x.ps1", "x.py", "x.js", "x.html"] {
        assert!(allowed_file("ocr-line", s).is_err());
    }
    assert!(allowed_file("external", "connector.xpi").is_ok());
}
#[test]
fn only_reviewed_https_origins_are_allowed() {
    for url in [
        "http://github.com/a",
        "https://localhost/a",
        "https://127.0.0.1/a",
        "https://github.com.evil.test/a",
        "https://user:password@github.com/a",
        "file:///a",
    ] {
        assert!(https(url).is_err(), "{url}");
    }
    assert!(https("https://github.com/wpy-sketch/ResearchClip/releases").is_ok());
}
#[test]
fn unavailable_production_catalog_never_reports_published() {
    let t = tempfile::tempdir().unwrap();
    let s = PackageManager::new(t.path().join("packages"), trust()).unwrap();
    assert!(!s.snapshot().unwrap().published);
    assert!(s.refresh().is_err());
    assert!(Arc::new(s).start("model.ocr-line", true).is_err());
}
#[test]
fn compiled_manifest_needs_no_keys_and_never_trusts_an_edited_local_catalog_or_proof() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("packages");
    let service = PackageManager::production(root.clone()).unwrap();
    let state = service.snapshot().unwrap();
    assert_eq!(state.catalog_mode, "BUILTIN");
    assert!(!state.published);
    let catalog = state.catalog.unwrap();
    assert_eq!(catalog.packages.len(), 2);
    assert!(
        catalog
            .packages
            .iter()
            .all(|p| ["model.ocr-line", "plugin.zotero-connector"].contains(&p.id.as_str()))
    );
    let package = &catalog.packages[0];
    let proof = service.builtin_proof.as_ref().unwrap();
    service.authenticate_package(package, proof).unwrap();
    for field in ["url", "hash", "version"] {
        let mut changed = package.clone();
        match field {
            "url" => changed.files[0].url = "https://github.com/evil/arbitrary".into(),
            "hash" => changed.files[0].sha256 = "a".repeat(64),
            _ => changed.version = "99.0.0".into(),
        };
        assert!(service.authenticate_package(&changed, proof).is_err());
    }
    let mut forged = proof.clone();
    forged.payload.push(' ');
    assert!(service.authenticate_package(package, &forged).is_err());
    // Local cache metadata cannot replace the exact compiled definition after restart.
    fs::write(
        root.join("catalog.json"),
        b"{\"payload\":\"{}\",\"signature\":\"BUILTIN_MANIFEST_V1\"}",
    )
    .unwrap();
    drop(service);
    let restored = PackageManager::production(root.clone()).unwrap();
    restored.refresh().unwrap();
    assert!(restored.snapshot().unwrap().catalog.unwrap().packages == catalog.packages);
    assert!(
        fs::read(root.join("catalog.json"))
            .unwrap()
            .starts_with(b"{\"payload\"")
    );
}
#[test]
fn future_registry_is_preserved_and_not_reset() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("packages");
    fs::create_dir(&root).unwrap();
    let text = b"{\"schemaVersion\":99,\"highestSequence\":0,\"installed\":[]}";
    fs::write(root.join("registry.json"), text).unwrap();
    assert!(PackageManager::new(root.clone(), trust()).is_err());
    assert_eq!(fs::read(root.join("registry.json")).unwrap(), text);
}
#[test]
fn bad_signature_and_expired_catalog_are_rejected() {
    let e = Envelope {
        payload: "{}".into(),
        signature: "not a signature".into(),
    };
    assert!(verify_catalog(&e, "not a key", 0).is_err());
    let c = Catalog {
        schema_version: 1,
        sequence: 1,
        expires_at: 1,
        packages: vec![],
    };
    assert!(validate(&c, now(), 0, "4.4.0").is_err());
    let c = Catalog {
        expires_at: now() + 100,
        sequence: 1,
        ..c
    };
    assert!(validate(&c, now(), 2, "4.4.0").is_err());
}
#[test]
#[ignore = "requires explicitly downloaded SHA-verified PP-OCR model files; runs real native CPU inference"]
fn real_onnx_cpu_ocr_self_test_and_offline_reopen() {
    let model_root =
        std::env::var_os("RESEARCHCLIP_PACKAGE_QA_MODELS").expect("explicit model fixture root");
    ocr_model::configure_runtime(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"))
        .unwrap();
    let path = PathBuf::from(model_root);
    ocr_model::self_test(&path).unwrap();
    let text = ocr_model::recognize(
        &path,
        include_bytes!("../../../../assets/ocr-self-test.png"),
    )
    .unwrap();
    assert!(text.contains("2026"));
    ocr_model::self_test(&path).unwrap();
    let chinese = ocr_model::recognize(
        &path,
        include_bytes!("../../../../assets/ocr-self-test-zh.png"),
    )
    .unwrap();
    assert_eq!(chinese.replace(' ', ""), "中文识别测试");
}
fn envelope() -> Envelope {
    serde_json::from_str(include_str!("fixtures/catalog-v1.json")).unwrap()
}
fn signed() -> Catalog {
    let e = envelope();
    let c: Catalog = serde_json::from_str(&e.payload).unwrap();
    verify_catalog_at(
        &e,
        include_str!("fixtures/catalog-test.pub").trim(),
        0,
        c.expires_at - 100,
    )
    .unwrap()
}
fn package() -> Package {
    signed()
        .packages
        .into_iter()
        .find(|p| p.adapter == "data")
        .unwrap()
}
fn trusted() -> Trust {
    let mut t = trust();
    t.catalog_public_key = Some(include_str!("fixtures/catalog-test.pub").trim().into());
    t
}
fn signed_manager(root: PathBuf) -> PackageManager {
    let s = PackageManager::new(root, trusted()).unwrap();
    {
        let mut state = s.state().unwrap();
        state.catalog = Some(signed());
        state.envelope = Some(envelope());
    }
    s
}
fn update_proof(s: &PackageManager) {
    let e: Envelope = serde_json::from_str(include_str!("fixtures/catalog-v2.json")).unwrap();
    let c = serde_json::from_str(&e.payload).unwrap();
    let mut state = s.state().unwrap();
    state.catalog = Some(c);
    state.envelope = Some(e);
}
fn seed(service: &PackageManager, p: &Package) {
    fs::write(
        service
            .root
            .join("downloads")
            .join(format!("{}.part", p.files[0].sha256)),
        include_bytes!("fixtures/readme.md"),
    )
    .unwrap();
}
#[test]
fn copied_test_installation_is_not_trusted_by_an_unconfigured_production_root() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("packages");
    let s = signed_manager(root.clone());
    let p = package();
    seed(&s, &p);
    s.install(&p).unwrap();
    let file = s
        .root
        .join("installed")
        .join(&s.snapshot().unwrap().registry.installed[0].directory)
        .join("readme.md");
    drop(s);
    let s = PackageManager::new(root, trust()).unwrap();
    s.revalidate_installed().unwrap();
    let entry = s.snapshot().unwrap().registry.installed.remove(0);
    assert_eq!(entry.readiness, "UNTRUSTED_INSTALLATION");
    assert!(!entry.enabled);
    assert!(s.control(&p.id, "enable").is_err());
    assert!(s.control(&p.id, "uninstall").is_err());
    assert_eq!(download::hash(&file).unwrap(), p.files[0].sha256);
    let mut registry = s.snapshot().unwrap().registry;
    registry.installed[0].directory = "other-valid-directory".into();
    save(&s.root, "registry.json", &registry).unwrap();
    assert!(PackageManager::new(s.root.clone(), trusted()).is_err());
    assert_eq!(download::hash(&file).unwrap(), p.files[0].sha256);
}
#[test]
fn uninstall_retains_unexpected_files_and_a_retryable_journal() {
    let t = tempfile::tempdir().unwrap();
    let s = signed_manager(t.path().join("packages"));
    let p = package();
    seed(&s, &p);
    s.install(&p).unwrap();
    let dir = s
        .root
        .join("installed")
        .join(&s.snapshot().unwrap().registry.installed[0].directory);
    let extra = dir.join("user-extra.txt");
    fs::write(&extra, "keep me").unwrap();
    assert!(s.control(&p.id, "uninstall").is_err());
    assert_eq!(
        s.snapshot().unwrap().registry.installed[0].readiness,
        "UNINSTALL_FAILED"
    );
    assert_eq!(fs::read_to_string(&extra).unwrap(), "keep me");
    fs::remove_file(extra).unwrap();
    s.control(&p.id, "uninstall").unwrap();
    assert!(s.snapshot().unwrap().registry.installed.is_empty());
}
#[test]
fn real_signed_catalog_tampering_wrong_key_and_sequence_rollback_fail() {
    let mut e = envelope();
    assert_eq!(signed().packages.len(), 2);
    e.payload.push(' ');
    assert!(
        verify_catalog_at(
            &e,
            include_str!("fixtures/catalog-test.pub").trim(),
            0,
            now()
        )
        .is_err()
    );
    assert!(
        verify_catalog(
            &envelope(),
            "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3",
            0
        )
        .is_err()
    );
    let c = signed();
    assert!(validate(&c, c.expires_at - 100, 2, "4.4.0").is_err());
}
#[test]
fn native_code_members_and_dependency_cycles_fail() {
    let mut wrong_kind = signed();
    wrong_kind.packages[0].kind = "EXTERNAL_PLUGIN".into();
    assert!(validate(&wrong_kind, wrong_kind.expires_at - 100, 0, "4.4.0").is_err());
    let mut p = package();
    p.files[0].path = "payload.dll".into();
    let mut c = signed();
    c.packages = vec![p];
    assert!(validate(&c, c.expires_at - 100, 0, "4.4.0").is_err());
    let mut c = signed();
    c.packages[0].dependencies = vec![c.packages[1].id.clone()];
    c.packages[1].dependencies = vec![c.packages[0].id.clone()];
    assert!(validate(&c, c.expires_at - 100, 0, "4.4.0").is_err());
}
#[test]
fn install_disable_enable_update_restart_and_uninstall_keep_user_files() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("packages");
    let user = t.path().join("research.md");
    fs::write(&user, "user-owned").unwrap();
    let s = signed_manager(root.clone());
    let mut p = package();
    seed(&s, &p);
    s.install(&p).unwrap();
    assert_eq!(
        s.snapshot().unwrap().registry.installed[0].readiness,
        "INSTALLED"
    );
    s.control(&p.id, "disable").unwrap();
    assert!(!s.snapshot().unwrap().registry.installed[0].enabled);
    s.control(&p.id, "enable").unwrap();
    p.version = "1.0.1".into();
    update_proof(&s);
    s.install(&p).unwrap();
    let after = s.snapshot().unwrap();
    assert_eq!(after.registry.retired.len(), 1);
    let previous = s
        .root
        .join("installed")
        .join(&after.registry.retired[0].directory);
    assert!(previous.is_dir());
    drop(s);
    let s = PackageManager::new(root, trusted()).unwrap();
    s.revalidate_installed().unwrap();
    assert_eq!(
        s.snapshot().unwrap().registry.installed[0].package.version,
        "1.0.1"
    );
    s.control(&p.id, "uninstall").unwrap();
    assert!(s.snapshot().unwrap().registry.installed.is_empty());
    assert!(!previous.exists());
    assert_eq!(fs::read_to_string(user).unwrap(), "user-owned");
}
#[test]
fn corrupt_update_and_incompatible_versions_preserve_old_installation() {
    let t = tempfile::tempdir().unwrap();
    let s = signed_manager(t.path().join("packages"));
    let mut p = package();
    seed(&s, &p);
    s.install(&p).unwrap();
    p.version = "1.0.1".into();
    update_proof(&s);
    fs::write(
        s.root
            .join("downloads")
            .join(format!("{}.part", p.files[0].sha256)),
        b"corrupt",
    )
    .unwrap();
    assert!(s.commit_cached_install(&p).is_err());
    let state = s.snapshot().unwrap();
    assert_eq!(state.registry.installed[0].package.version, "1.0.0");
    p.app_version = ">=99.0.0".into();
    assert!(s.compatible(&p, &state.registry).is_err());
}
#[test]
fn cancellation_before_download_makes_no_network_request() {
    let t = tempfile::tempdir().unwrap();
    let cancelled = AtomicBool::new(true);
    assert!(
        download::download(&package().files[0], t.path(), &cancelled, |_| panic!(
            "no progress after cancellation"
        ))
        .unwrap_err()
        .contains("取消")
    );
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
}
#[test]
fn registry_commit_failure_preserves_old_version_and_retry_recovers_intent() {
    let t = tempfile::tempdir().unwrap();
    let s = signed_manager(t.path().join("packages"));
    let mut p = package();
    seed(&s, &p);
    s.install(&p).unwrap();
    p.version = "1.0.1".into();
    update_proof(&s);
    let registry = s.root.join("registry.json");
    let preserved = s.root.join("registry-test-preserved.json");
    fs::rename(&registry, &preserved).unwrap();
    fs::create_dir(&registry).unwrap();
    assert!(s.commit_cached_install(&p).is_err());
    assert_eq!(
        s.snapshot().unwrap().registry.installed[0].package.version,
        "1.0.0"
    );
    let intent = s.root.join(format!(
        "intent-{}-{}-{}.json",
        p.id,
        p.version,
        &p.files[0].sha256[..12]
    ));
    assert!(intent.is_file());
    fs::remove_dir(&registry).unwrap();
    fs::rename(preserved, &registry).unwrap();
    s.commit_cached_install(&p).unwrap();
    assert_eq!(
        s.snapshot().unwrap().registry.installed[0].package.version,
        "1.0.1"
    );
    assert!(!intent.exists());
    assert_eq!(s.snapshot().unwrap().registry.retired.len(), 1);
}
#[test]
#[ignore = "real reviewed HTTPS download cancellation and restart; host may safely return 200 instead of Range"]
fn real_https_interruption_and_resume_or_safe_restart() {
    let t = tempfile::tempdir().unwrap();
    // Exercise the same downloader against an existing GitHub asset. The model
    // host can be unavailable independently; do not replace failed transfers
    // with fabricated bytes or weaken partial-file and hash assertions.
    let candidate: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../docs/release/package-catalog.candidate.json"
    ))
    .unwrap();
    let p: Package = serde_json::from_value(
        candidate["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "plugin.browser-clipper")
            .unwrap()
            .clone(),
    )
    .unwrap();
    let asset = &p.files[0];
    let cancel = AtomicBool::new(false);
    let interruption = download::download(asset, t.path(), &cancel, |n| {
        if n >= 65536 {
            cancel.store(true, Ordering::Release);
        }
    });
    assert!(
        cancel.load(Ordering::Acquire),
        "network failed before cancellation could be tested: {interruption:?}"
    );
    assert!(interruption.is_err());
    let path = t.path().join(format!("{}.part", asset.sha256));
    let partial = fs::metadata(&path).unwrap().len();
    assert!(partial > 0 && partial < asset.size);
    cancel.store(false, Ordering::Release);
    let mut initial = None;
    let completed = download::download(asset, t.path(), &cancel, |n| {
        initial.get_or_insert(n);
    })
    .unwrap();
    assert!(initial == Some(partial) || initial == Some(0));
    download::verify(&completed, asset.size, &asset.sha256).unwrap();
    println!(
        "real partial bytes={partial}; restart initial bytes={}",
        initial.unwrap()
    );
}
#[test]
fn safe_zip_extraction_rejects_unlisted_members_and_path_traversal() {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    let t = tempfile::tempdir().unwrap();
    let mut p = package();
    let path = t.path().join("archive.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
    zip.start_file("../escape.md", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"x").unwrap();
    zip.finish().unwrap();
    let bytes = fs::read(&path).unwrap();
    let sha = format!("{:x}", Sha256::digest(&bytes));
    fs::rename(path, t.path().join(format!("{sha}.part"))).unwrap();
    p.files = vec![Asset {
        path: "archive.zip".into(),
        url: p.files[0].url.clone(),
        size: bytes.len() as u64,
        sha256: sha,
        entries: vec![Entry {
            path: "safe.md".into(),
            size: 1,
            sha256: format!("{:x}", Sha256::digest(b"x")),
        }],
    }];
    assert!(PackageInstaller::stage(&p, t.path(), &t.path().join("stage")).is_err());
    assert!(!t.path().parent().unwrap().join("escape.md").exists());
}
#[test]
fn downloaded_external_plugin_cannot_be_marked_enabled_in_target_host() {
    let t = tempfile::tempdir().unwrap();
    let s = PackageManager::new(t.path().join("packages"), trust()).unwrap();
    let p = package();
    assert!(s.control("plugin.zotero-connector", "enable").is_err());
    assert!(s.installed_file(&p.id).is_err());
}
#[test]
#[ignore = "requires real reviewed HTTPS model host; no synthetic recognition or package bytes"]
fn real_https_model_download() {
    let t = tempfile::tempdir().unwrap();
    let p = signed()
        .packages
        .into_iter()
        .find(|p| p.id == "model.ocr-line")
        .unwrap();
    let cancel = AtomicBool::new(false);
    for f in p.files {
        let path = download::download(&f, t.path(), &cancel, |_| {}).unwrap();
        download::verify(&path, f.size, &f.sha256).unwrap();
    }
}
#[test]
#[ignore = "requires explicit verified aligned CLIP model fixture; actual CPU text/image inference"]
fn real_native_semantic_pair() {
    let root = PathBuf::from(
        std::env::var_os("RESEARCHCLIP_PACKAGE_QA_SEMANTIC").expect("explicit semantic fixtures"),
    );
    ocr_model::configure_runtime(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"))
        .unwrap();
    let manifest: serde_json::Value =
        serde_json::from_str(crate::services::embeddings::MANIFEST).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        download::verify(
            &root.join(f["name"].as_str().unwrap()),
            f["size"].as_u64().unwrap(),
            f["sha256"].as_str().unwrap(),
        )
        .unwrap();
    }
    semantic_model::self_test(&root).unwrap();
}

#[test]
fn major_version_upgrade_retains_signed_v4_files_but_requires_a_compatible_package_update() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("packages");
    let legacy: Envelope =
        serde_json::from_str(include_str!("fixtures/catalog-v1-v440.json")).unwrap();
    let catalog: Catalog = serde_json::from_str(&legacy.payload).unwrap();
    let package = catalog
        .packages
        .iter()
        .find(|p| p.adapter == "data")
        .unwrap()
        .clone();
    let manager = PackageManager::new(root.clone(), trusted()).unwrap();
    let directory = format!(
        "{}-{}-{}",
        package.id,
        package.version,
        &package.files[0].sha256[..12]
    );
    let files = root.join("installed").join(&directory);
    fs::create_dir(&files).unwrap();
    fs::write(
        files.join("readme.md"),
        include_bytes!("fixtures/readme.md"),
    )
    .unwrap();
    let registry = PackageRegistry {
        installed: vec![Installed {
            package: package.clone(),
            directory,
            enabled: true,
            readiness: "READY".into(),
            catalog_proof: Some(legacy),
            installed_at: now(),
        }],
        ..PackageRegistry::default()
    };
    save(&root, "registry.json", &registry).unwrap();
    drop(manager);
    let manager = PackageManager::new(root.clone(), trusted()).unwrap();
    manager.revalidate_installed().unwrap();
    let snapshot = manager.snapshot().unwrap();
    assert_eq!(
        snapshot.registry.installed[0].readiness,
        "APP_UPDATE_REQUIRED"
    );
    assert!(!snapshot.registry.installed[0].enabled);
    assert!(manager.control(&package.id, "enable").is_err());
    assert_eq!(
        download::hash(&files.join("readme.md")).unwrap(),
        package.files[0].sha256
    );
}
