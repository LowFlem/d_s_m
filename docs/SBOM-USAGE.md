# DSM SBOM Usage Guide

## What is a Software Bill of Materials (SBOM)?

An SBOM is a comprehensive inventory of all software components, dependencies, and libraries used in your application. Think of it as an "ingredients list" for software - it tells you exactly what's inside your application and where it came from.

## Why SBOMs Matter for DSM

1. **Security**: Identify vulnerable components before they're exploited
2. **Compliance**: Meet regulatory requirements (Executive Order 14028, NIST guidelines)
3. **Supply Chain**: Track all dependencies and their provenance
4. **Transparency**: Know exactly what cryptographic libraries protect your quantum-resistant features
5. **Audit Trail**: Essential for post-quantum cryptography certification

## DSM SBOM System Overview

Our SBOM system generates comprehensive inventories for:
- **Rust Core**: All cryptographic libraries (ML-KEM, SPHINCS+, Blake3)
- **Storage Node**: Network and persistence dependencies  
- **Frontend**: React/TypeScript dependencies
- **Android**: Gradle/Kotlin dependencies
- **Consolidated**: Complete system view with 850+ components

## Quick Start: Generate Your First SBOM

```bash
# Generate complete SBOM suite
./scripts/generate-sbom.sh

# View consolidated SBOM (all components)
cat sbom/dsm-consolidated-*.sbom.json | jq '.metadata.component'

# Check vulnerability status
cat sbom/vulnerability-report-*.json | jq '.summary'
```

## Common SBOM Use Cases

### 1. Security Vulnerability Scanning

```bash
# Generate fresh SBOM with latest dependency data
./scripts/generate-sbom.sh

# Check for known vulnerabilities
cat sbom/rust-audit-*.json | jq '.vulnerabilities.list[]'
cat sbom/npm-audit-*.json | jq '.vulnerabilities'

# Review high-severity issues
jq '.vulnerabilities.list[] | select(.advisory.severity == "high")' sbom/rust-audit-*.json
```

### 2. Compliance Reporting

```bash
# Generate compliance report for auditors
./scripts/generate-sbom.sh
cat sbom/compliance-report-*.md

# Extract license information
jq '.components[] | select(.licenses) | {name: .name, licenses: .licenses}' sbom/dsm-consolidated-*.sbom.json

# Post-quantum crypto verification
jq '.components[] | select(.name | contains("ml-kem") or contains("sphincs") or contains("blake3"))' sbom/dsm-consolidated-*.sbom.json
```

### 3. Supply Chain Analysis

```bash
# Find all cryptographic dependencies
jq '.components[] | select(.name | test("crypto|cipher|hash|random|kem|sphincs"))' sbom/dsm-consolidated-*.sbom.json

# Check dependency origins
jq '.components[] | {name: .name, supplier: .supplier, version: .version}' sbom/dsm-consolidated-*.sbom.json | head -20

# Identify potential supply chain risks
jq '.components[] | select(.supplier.name == null or .supplier.name == "")' sbom/dsm-consolidated-*.sbom.json
```

### 4. CI/CD Integration

```bash
# Add to your CI pipeline
./scripts/generate-sbom.sh
# Check if new vulnerabilities introduced
if [ -s "sbom/vulnerability-report-*.json" ]; then
    echo "New vulnerabilities found - review before deployment"
    exit 1
fi
```

## Understanding SBOM Output Files

After running `./scripts/generate-sbom.sh`, you'll get:

```
sbom/
├── dsm-consolidated-TIMESTAMP.sbom.json    # Complete system SBOM (851 components)
├── dsm-core-TIMESTAMP.sbom.json           # Rust core dependencies (620KB)
├── dsm-storage-node-TIMESTAMP.sbom.json   # Storage node dependencies (428KB)
├── frontend-TIMESTAMP.sbom.json           # React/Node.js dependencies
├── android-TIMESTAMP.sbom.json            # Android/Kotlin dependencies
├── vulnerability-report-TIMESTAMP.json    # Security scan results
├── compliance-report-TIMESTAMP.md         # Regulatory compliance status
└── metadata-TIMESTAMP.json               # Generation metadata
```

## Key SBOM Fields Explained

```json
{
  "bomFormat": "CycloneDX",           // Industry standard format
  "specVersion": "1.4",               // Specification version
  "serialNumber": "urn:uuid:...",     // Unique identifier
  "metadata": {
    "component": {
      "type": "application",          // Component type
      "name": "DSM Protocol Suite",   // Your application
      "version": "2.1.0"             // Version tracking
    }
  },
  "components": [                     // All dependencies
    {
      "type": "library",
      "name": "ml-kem",               // Post-quantum crypto
      "version": "0.2.0",
      "supplier": {
        "name": "RustCrypto"          // Trusted supplier
      },
      "hashes": [{                    // Integrity verification
        "alg": "SHA-256",
        "content": "abc123..."
      }]
    }
  ]
}
```

## Advanced SBOM Analysis

### Find Security-Critical Components

```bash
# Post-quantum cryptography components
jq '.components[] | select(.name | test("ml-kem|sphincs|kyber|blake3|chacha20"))' sbom/dsm-consolidated-*.sbom.json

# Memory-unsafe languages (should be minimal in DSM)
jq '.components[] | select(.name | test("^lib.*\\.so$|^.*\\.dll$"))' sbom/dsm-consolidated-*.sbom.json

# Network/TLS components
jq '.components[] | select(.name | test("tls|ssl|http|net|tcp"))' sbom/dsm-consolidated-*.sbom.json
```

### Track License Compliance

```bash
# Extract all unique licenses
jq -r '.components[].licenses[]?.license.id // .components[].licenses[]?.license.name // "Unknown"' sbom/dsm-consolidated-*.sbom.json | sort | uniq -c

# Check for GPL contamination (not allowed in DSM)
jq '.components[] | select(.licenses[]?.license.id | test("GPL"))' sbom/dsm-consolidated-*.sbom.json

# Verify MIT/Apache-2.0 compliance
jq '.components[] | select(.licenses[]?.license.id | test("MIT|Apache"))' sbom/dsm-consolidated-*.sbom.json | wc -l
```

### Monitor Component Freshness

```bash
# Components without version info (potential risk)
jq '.components[] | select(.version == null or .version == "")' sbom/dsm-consolidated-*.sbom.json

# Compare SBOMs over time (run after updates)
diff <(jq '.components[].name' sbom/dsm-consolidated-OLD.sbom.json | sort) \
     <(jq '.components[].name' sbom/dsm-consolidated-NEW.sbom.json | sort)
```

## Integration with Security Tools

### Vulnerability Databases

The SBOM integrates with:
- **RustSec Advisory DB**: Rust-specific vulnerabilities
- **NPM Security Advisories**: Node.js vulnerabilities  
- **NVD (National Vulnerability Database)**: CVE tracking
- **OSV (Open Source Vulnerabilities)**: Cross-ecosystem coverage

### Enterprise Security Platforms

SBOMs work with:
- **Snyk**: Import SBOM for continuous monitoring
- **JFrog Xray**: Supply chain security scanning
- **Sonatype Nexus**: Component policy enforcement
- **GitHub Security**: Dependency vulnerability alerts

## Best Practices

### 1. Regular Generation
```bash
# Generate SBOMs on every build
echo "./scripts/generate-sbom.sh" >> .git/hooks/pre-commit

# Automated CI/CD integration
# Add to GitHub Actions, GitLab CI, etc.
```

### 2. Version Control SBOMs
```bash
# Track SBOM changes over time
git add sbom/dsm-consolidated-*.sbom.json
git commit -m "Update SBOM: $(date)"
```

### 3. Stakeholder Communication
```bash
# Generate readable compliance report
./scripts/generate-sbom.sh
cat sbom/compliance-report-*.md > COMPLIANCE.md
```

### 4. Incident Response
```bash
# When vulnerability disclosed:
# 1. Generate fresh SBOM
./scripts/generate-sbom.sh

# 2. Check if affected
jq '.components[] | select(.name == "vulnerable-component")' sbom/dsm-consolidated-*.sbom.json

# 3. Update dependencies
# 4. Regenerate SBOM
# 5. Verify fix
```

## DSM-Specific SBOM Considerations

### Quantum-Resistant Verification
```bash
# Ensure post-quantum crypto is present
jq '.components[] | select(.name | test("ml-kem|sphincs|blake3"))' sbom/dsm-consolidated-*.sbom.json | length
# Should return 3 or more components
```

### No-Fallback Verification
```bash
# Verify no fallback crypto libraries
jq '.components[] | select(.name | test("openssl|mbedtls|ring"))' sbom/dsm-consolidated-*.sbom.json
# Should return empty (DSM uses only post-quantum crypto)
```

### Memory Safety Verification
```bash
# Verify Rust safety
jq '.components[] | select(.type == "library" and (.name | test("^[^-]+-sys$")))' sbom/dsm-consolidated-*.sbom.json | wc -l
# Should be minimal (only for JNI boundary)
```

## Troubleshooting

### SBOM Generation Fails
```bash
# Check dependencies
cargo install cargo-cyclonedx
npm install -g @cyclonedx/bom

# Verify workspace structure
ls -la dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml
```

### Missing Components
```bash
# Regenerate with verbose output
RUST_LOG=debug ./scripts/generate-sbom.sh

# Check individual component generation
cd dsm_client/decentralized_state_machine/dsm_sdk
cargo cyclonedx --format json
```

### Large SBOM Files
```bash
# Compress for storage
gzip sbom/dsm-consolidated-*.sbom.json

# Filter critical components only
jq '.components[] | select(.name | test("ml-kem|sphincs|dsm"))' sbom/dsm-consolidated-*.sbom.json
```

## Conclusion

SBOMs are essential for:
1. **Security**: Know your attack surface
2. **Compliance**: Meet regulatory requirements  
3. **Trust**: Verify post-quantum crypto implementation
4. **Maintenance**: Track dependency health

The DSM SBOM system provides comprehensive visibility into all 851 components, ensuring the quantum-resistant architecture maintains its security guarantees throughout the software supply chain.

For more information, see:
- `./scripts/generate-sbom.sh` - Generation script
- `sbom/compliance-report-*.md` - Regulatory compliance status
- `docs/SECURITY.md` - Overall security documentation
