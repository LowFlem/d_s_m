# DSM SBOM Usage Guide

## Overview
This guide covers how to properly use, read, and analyze the Software Bill of Materials (SBOM) generated for the DSM project.

## Recommended VSCode Extensions

Install these extensions for optimal SBOM viewing and analysis:

### 1. JSON Tools
- **Extension**: `ms-vscode.vscode-json`
- **Purpose**: Enhanced JSON viewing with syntax highlighting and validation
- **Usage**: Automatically formats and validates SBOM JSON files

### 2. SARIF Viewer
- **Extension**: `ms-sarif.sarif-viewer`
- **Purpose**: View security scan results in SARIF format
- **Usage**: Displays vulnerability reports with context

### 3. YAML Support
- **Extension**: `redhat.vscode-yaml`
- **Purpose**: YAML syntax support for configuration files
- **Usage**: View YAML-formatted SBOMs and configs

### 4. CycloneDX VSCode Extension
- **Extension**: `cyclonedx.cyclonedx-vscode`
- **Purpose**: Native CycloneDX SBOM viewing and analysis
- **Usage**: Provides tree view, dependency analysis, and vulnerability integration

## Command Line Tools

### 1. CycloneDX CLI
```bash
# Install CycloneDX CLI for SBOM analysis
npm install -g @cyclonedx/cyclonedx-cli

# Validate SBOM
cyclonedx validate --input-file dsm-consolidated-*.sbom.json

# Convert formats
cyclonedx convert --input-file dsm-core.sbom.json --output-format xml

# Diff SBOMs
cyclonedx diff --input-file v1.sbom.json --another-file v2.sbom.json
```

### 2. SPDX Tools
```bash
# Install SPDX tools
pip install spdx-tools

# Convert CycloneDX to SPDX
spdx-tools convert --from cyclonedx --to spdx --input dsm-consolidated.sbom.json --output dsm.spdx
```

### 3. JQ for JSON Processing
```bash
# Install jq if not available
brew install jq  # macOS
apt-get install jq  # Ubuntu

# Query SBOM data
jq '.components[] | select(.type == "library") | .name' dsm-consolidated.sbom.json
jq '.vulnerabilities[] | select(.severity == "high")' vulnerability-report.json
```

## Reading SBOM Reports

### 1. Consolidated SBOM Structure
```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.4",
  "serialNumber": "urn:uuid:...",
  "version": 1,
  "metadata": {
    "timestamp": "2025-07-02T10:31:49Z",
    "tools": [...],
    "component": {
      "type": "application",
      "name": "DSM Protocol Suite",
      "version": "2.1.0"
    }
  },
  "components": [...],  // 851 components
  "vulnerabilities": [...],
  "dependencies": [...]
}
```

### 2. Key Sections to Review

#### Components Section
- Lists all dependencies with versions
- Includes license information
- Shows component relationships

#### Vulnerabilities Section
- Lists known security issues
- Provides severity ratings
- Links to remediation advice

#### Dependencies Section
- Shows dependency relationships
- Helps identify supply chain risks

## Analysis Workflows

### 1. Security Review
```bash
# Check for high-severity vulnerabilities
cat sbom/vulnerability-report-*.json | jq '.vulnerabilities[] | select(.severity == "high")'

# List post-quantum crypto components
cat sbom/dsm-consolidated-*.sbom.json | jq '.components[] | select(.name | contains("ml-kem") or contains("sphincs") or contains("blake3"))'

# Check license compliance
cat sbom/dsm-consolidated-*.sbom.json | jq '.components[] | .licenses[]?.license.name' | sort | uniq -c
```

### 2. Dependency Analysis
```bash
# Count components by type
cat sbom/dsm-consolidated-*.sbom.json | jq '.components[] | .type' | sort | uniq -c

# Find components without licenses
cat sbom/dsm-consolidated-*.sbom.json | jq '.components[] | select(.licenses == null) | .name'

# Check for outdated dependencies
cat sbom/rust-audit-*.json | jq '.warnings[]'
```

### 3. Compliance Checking
```bash
# Generate compliance report
./scripts/generate-sbom.sh

# View compliance status
cat sbom/compliance-report-*.md
```

## Automated Workflows

### 1. CI/CD Integration
Add to `.github/workflows/sbom.yml`:
```yaml
name: SBOM Generation
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Generate SBOM
        run: ./scripts/generate-sbom.sh
      - name: Upload SBOMs
        uses: actions/upload-artifact@v3
        with:
          name: sbom-reports
          path: sbom/
```

### 2. Security Scanning
```bash
# Automated vulnerability scanning
./scripts/generate-sbom.sh
cat sbom/vulnerability-report-*.json | jq '.vulnerabilities[] | select(.severity == "critical" or .severity == "high")'
```

## DSM-Specific SBOM Analysis

### 1. Quantum-Resistant Components
```bash
# Verify post-quantum crypto dependencies
cat sbom/dsm-consolidated-*.sbom.json | jq '.components[] | select(.name | contains("ml-kem") or contains("sphincs") or contains("blake3") or contains("chacha20")) | {name: .name, version: .version, type: .type}'
```

### 2. Rust Memory Safety
```bash
# Check for unsafe Rust crates
cat sbom/dsm-core-*.sbom.json | jq '.components[] | select(.name | contains("unsafe") or contains("libc") or contains("winapi")) | {name: .name, version: .version}'
```

### 3. JNI Dependencies
```bash
# List JNI-related components
cat sbom/dsm-consolidated-*.sbom.json | jq '.components[] | select(.name | contains("jni") or contains("android")) | {name: .name, version: .version}'
```

## Best Practices

### 1. Regular SBOM Updates
- Generate SBOMs on every release
- Compare SBOMs between versions
- Track dependency changes over time

### 2. Security Monitoring
- Set up automated vulnerability scanning
- Monitor for new vulnerabilities in dependencies
- Establish remediation timelines

### 3. License Compliance
- Track all software licenses
- Ensure compatibility with project license
- Document license obligations

### 4. Supply Chain Security
- Verify component authenticity
- Monitor for suspicious dependencies
- Implement dependency pinning

## Troubleshooting

### Common Issues

1. **Large SBOM Files**: Use `jq` for filtering and processing
2. **Missing Dependencies**: Check package-lock.json and Cargo.lock
3. **Vulnerability False Positives**: Review and document accepted risks
4. **License Conflicts**: Consult legal team for resolution

### Getting Help

- CycloneDX Documentation: https://cyclonedx.org/docs/
- SPDX Specification: https://spdx.github.io/spdx-spec/
- NIST SBOM Resources: https://www.nist.gov/sbom

## DSM Protocol Compliance

The DSM SBOM system ensures:
- ✅ All business logic in Rust core is tracked
- ✅ Post-quantum cryptography dependencies verified
- ✅ JNI bridge components documented
- ✅ No fallback dependencies (blueprint compliant)
- ✅ Forward-compatible architecture maintained

## Quick Reference Commands

```bash
# Generate complete SBOM suite
./scripts/generate-sbom.sh

# View consolidated SBOM
cat sbom/dsm-consolidated-*.sbom.json | jq .

# Check vulnerabilities
cat sbom/vulnerability-report-*.json | jq '.vulnerabilities[] | select(.severity == "high" or .severity == "critical")'

# View compliance report
cat sbom/compliance-report-*.md

# Validate SBOM format
cyclonedx validate --input-file sbom/dsm-consolidated-*.sbom.json
```

---

*Generated for DSM Protocol v2.1 - Production Ready*
