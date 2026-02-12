#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use insta::assert_snapshot;

    #[test]
    fn test_show_missing_lines() {
        let json = include_str!("fixtures/show-missing-lines.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_show_missing_lines_complete() {
        let json = include_str!("fixtures/show-missing-lines-complete.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_show_missing_lines_multi_missing() {
        let json = include_str!("fixtures/show-missing-lines-multi-missing.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_no_test_coverage() {
        let json = include_str!("fixtures/no_test.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_real1_all() {
        let json = include_str!("fixtures/real1-all.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_with_branches() {
        let json = include_str!("fixtures/with-branches.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_all_covered() {
        let json = include_str!("fixtures/all-covered.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_malformed_json() {
        let result = llvm_cov_easy::analyze_and_format("not json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_snapshot!(err.to_string(), @"failed to parse coverage JSON: expected ident at line 1 column 2");
    }

    #[test]
    fn test_empty_data() {
        let json = r#"{"data":[],"type":"llvm.coverage.json.export","version":"2.0.1"}"#;
        let result = llvm_cov_easy::analyze_and_format(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_snapshot!(err.to_string(), @"coverage data is empty (no data entries)");
    }

    #[test]
    fn test_summary_only_no_segments() {
        // Version 3.1.0 format with only summary, no segments
        let json = include_str!("fixtures/summary-only.json");
        let output = llvm_cov_easy::analyze_and_format(json).unwrap();
        assert_snapshot!(output);
    }

    #[test]
    fn test_malformed_segment() {
        // Segment array has wrong number of elements — triggers Segment::deserialize error path
        let json = r#"{"data":[{"files":[{"filename":"src/lib.rs","segments":[[1,1,0]],"summary":{"lines":{"count":1,"covered":0,"percent":0.0},"regions":{"count":1,"covered":0,"percent":0.0},"functions":{"count":1,"covered":0,"percent":0.0}}}],"totals":{"lines":{"count":1,"covered":0,"percent":0.0},"regions":{"count":1,"covered":0,"percent":0.0},"functions":{"count":1,"covered":0,"percent":0.0}}}],"type":"llvm.coverage.json.export","version":"2.0.1"}"#;
        let result = llvm_cov_easy::analyze_and_format(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_branch() {
        // Branch array has wrong number of elements — triggers Branch::deserialize error path
        let json = r#"{"data":[{"files":[{"filename":"src/lib.rs","segments":[],"branches":[[1,1]],"summary":{"lines":{"count":1,"covered":0,"percent":0.0},"regions":{"count":1,"covered":0,"percent":0.0},"functions":{"count":1,"covered":0,"percent":0.0}}}],"totals":{"lines":{"count":1,"covered":0,"percent":0.0},"regions":{"count":1,"covered":0,"percent":0.0},"functions":{"count":1,"covered":0,"percent":0.0}}}],"type":"llvm.coverage.json.export","version":"2.0.1"}"#;
        let result = llvm_cov_easy::analyze_and_format(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_region() {
        // Region array has wrong number of elements — triggers Region::deserialize error path
        let json = r#"{"data":[{"files":[{"filename":"src/lib.rs","segments":[],"summary":{"lines":{"count":1,"covered":0,"percent":0.0},"regions":{"count":1,"covered":0,"percent":0.0},"functions":{"count":1,"covered":0,"percent":0.0}}}],"functions":[{"name":"foo","count":1,"filenames":["src/lib.rs"],"regions":[[1,1]],"branches":[]}],"totals":{"lines":{"count":1,"covered":0,"percent":0.0},"regions":{"count":1,"covered":0,"percent":0.0},"functions":{"count":1,"covered":0,"percent":0.0}}}],"type":"llvm.coverage.json.export","version":"2.0.1"}"#;
        let result = llvm_cov_easy::analyze_and_format(json);
        assert!(result.is_err());
    }
}
