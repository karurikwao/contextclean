$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$FixtureDir = Join-Path $Root "benchmarks/fixtures"
$ResultsJson = Join-Path $Root "benchmarks/results.json"
$ResultsMd = Join-Path $Root "benchmarks/results.md"
New-Item -ItemType Directory -Force -Path $FixtureDir | Out-Null

function Write-Utf8File {
    param(
        [string] $Path,
        [string[]] $Lines
    )
    [System.IO.File]::WriteAllText($Path, ($Lines -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
}

function New-HtmlScrapeFixture {
    param([string] $Path)
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add("<!doctype html>")
    $lines.Add("<html><head><title>Noisy docs export</title>")
    $lines.Add("<style>body{font-family:sans-serif}.ad{display:block}.modal{position:fixed}</style>")
    $lines.Add("<script>")
    for ($i = 1; $i -le 1400; $i++) {
        $lines.Add("window.analyticsQueue.push({event:'scroll', id:$i, cookie:'session-$i', viewport:'1920x1080', referrer:'campaign-$i'});")
    }
    $lines.Add("</script></head><body>")
    $lines.Add("<nav>Home Pricing Login Cookie preferences Accept all cookies Manage preferences</nav>")
    for ($i = 1; $i -le 260; $i++) {
        $lines.Add("<aside class='ad tracking-unit'>Advertisement slot $i sponsored content newsletter signup cookie preferences</aside>")
    }
    $lines.Add("<main>")
    $lines.Add("<h1>KEEP_HTML_ROOT_CAUSE: API setup troubleshooting guide</h1>")
    $lines.Add("<p>This generated fixture mimics a long documentation scrape with real article content surrounded by web chrome.</p>")
    $lines.Add("<table><tr><th>Configuration table</th><th>Preserved value</th></tr><tr><td>retry_timeout_ms</td><td>5000</td></tr></table>")
    $lines.Add("<pre><code>fn keep_code_block() { println!(`"KEEP_CODE_BLOCK`"); }</code></pre>")
    for ($i = 1; $i -le 520; $i++) {
        $lines.Add("<p>Step ${i}: preserve this deployment note about context budgets, failed requests, retry windows, and src/service_$i.ts:$($i + 20).</p>")
    }
    $lines.Add("</main>")
    $lines.Add("<footer>Newsletter signup Privacy choices All rights reserved</footer>")
    $lines.Add("</body></html>")
    Write-Utf8File -Path $Path -Lines $lines
}

function New-GhaFailureFixture {
    param([string] $Path)
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add("2026-07-04T10:00:00Z [group] npm install")
    for ($i = 1; $i -le 1100; $i++) {
        $lines.Add("2026-07-04T10:00:$($i % 60).000Z info added 481 packages in $i ms")
        $lines.Add("2026-07-04T10:00:$($i % 60).000Z info found 0 vulnerabilities")
    }
    for ($i = 1; $i -le 900; $i++) {
        $second = "{0:D2}" -f ($i % 60)
        $lines.Add("2026-07-04T10:22:$second`Z warn Connection timeout to database")
    }
    $lines.Add("FAIL packages/api/user.test.ts")
    $lines.Add("Unique failure:")
    $lines.Add("TypeError: Cannot read properties of undefined")
    $lines.Add("Final error summary: request failed after retries")
    for ($i = 1; $i -le 120; $i++) {
        $lines.Add("    at loadUser$i (/workspace/packages/api/src/user.ts:$($i + 40):13)")
        $lines.Add("    at main$i (/workspace/packages/api/src/main.ts:$($i + 8):1)")
    }
    for ($i = 1; $i -le 120; $i++) {
        $lines.Add("    at loadUser$i (/workspace/packages/api/src/user.ts:$($i + 40):13)")
        $lines.Add("    at main$i (/workspace/packages/api/src/main.ts:$($i + 8):1)")
    }
    Write-Utf8File -Path $Path -Lines $lines
}

function New-ProviderCiFixture {
    param([string] $Path)
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add("::group::install")
    for ($i = 1; $i -le 120; $i++) {
        $lines.Add("npm http fetch GET 200 https://registry.npmjs.org/package-$i $i ms")
        $lines.Add("npm WARN deprecated package-$i@1.0.0: fixture deprecation noise")
        $lines.Add("Progress: resolved $($i * 10), reused $($i * 9), downloaded 1, added 1")
        $lines.Add("Packages: +$i")
        $lines.Add("Downloaded crate_$i v1.0.$i")
        $lines.Add("Compiling crate_$i v1.0.$i")
        $lines.Add("tests/test_$i.py::test_ok PASSED [ 50%]")
        $lines.Add("#$i [internal] load build definition from Dockerfile")
        $lines.Add("#$i CACHED")
        $lines.Add("ok $i [chromium] › feature_$i.spec.ts:1:1 › passes (1.0s)")
    }
    $lines.Add("FAIL packages/api/user.test.ts")
    $lines.Add("tests/test_api.py::test_user FAILED [ 16%]")
    $lines.Add("Error: KEEP_PROVIDER_FAILURE")
    $lines.Add("Final error summary: provider test failed")
    Write-Utf8File -Path $Path -Lines $lines
}

function New-StackTraceFixture {
    param([string] $Path)
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add("Error: KEEP_STACK_ROOT payment reconciliation failed")
    $lines.Add("Caused by: UNIQUE_DATABASE_CONSTRAINT missing payment ledger row")
    for ($round = 1; $round -le 18; $round++) {
        for ($i = 1; $i -le 80; $i++) {
            $lines.Add("    at reconcileStep$i (/srv/app/src/reconcile/processor.ts:$($i + 10):17)")
        }
    }
    $lines.Add("Recovery frames kept after duplicate collapse:")
    for ($i = 1; $i -le 40; $i++) {
        $lines.Add("    at uniqueRecoveryFrame$i (/srv/app/src/reconcile/recovery.ts:$($i + 5):9)")
    }
    Write-Utf8File -Path $Path -Lines $lines
}

$htmlFixture = Join-Path $FixtureDir "html_scrape_large.html"
$ghaFixture = Join-Path $FixtureDir "github_actions_failure_large.log"
$providerFixture = Join-Path $FixtureDir "provider_ci_mix.log"
$stackFixture = Join-Path $FixtureDir "stack_trace_dump_large.log"

New-HtmlScrapeFixture -Path $htmlFixture
New-GhaFailureFixture -Path $ghaFixture
New-ProviderCiFixture -Path $providerFixture
New-StackTraceFixture -Path $stackFixture

$cargo = Join-Path $env:USERPROFILE ".cargo/bin/cargo.exe"
if (!(Test-Path $cargo)) {
    $cargo = "cargo"
}
& $cargo build -p contextclean-cli --release --locked

$ctxclean = Join-Path $Root "target/release/ctxclean.exe"
if (!(Test-Path $ctxclean)) {
    $ctxclean = Join-Path $Root "target/release/ctxclean"
}

$cases = @(
    @{
        Name = "HTML scrape"
        Fixture = "benchmarks/fixtures/html_scrape_large.html"
        Args = @("--mode", "standard", "--max-tokens", "5900")
        MustContain = @("KEEP_HTML_ROOT_CAUSE", "Configuration table", "KEEP_CODE_BLOCK")
        MustNotContain = @("window.analyticsQueue", "Advertisement slot", "Accept all cookies")
        Recommended = "ctxclean benchmarks/fixtures/html_scrape_large.html --mode standard --max-tokens 5900"
    },
    @{
        Name = "CI failure log"
        Fixture = "benchmarks/fixtures/github_actions_failure_large.log"
        Args = @("--mode", "aggressive", "--max-tokens", "3200")
        MustContain = @("FAIL packages/api/user.test.ts", "Unique failure:", "Final error summary")
        MustNotContain = @("added 481 packages", "found 0 vulnerabilities")
        Recommended = "ctxclean gha benchmarks/fixtures/github_actions_failure_large.log --max-tokens 3200 --format markdown"
    },
    @{
        Name = "Provider CI mix"
        Fixture = "benchmarks/fixtures/provider_ci_mix.log"
        Args = @("--mode", "aggressive", "--max-tokens", "1600")
        MustContain = @("FAIL packages/api/user.test.ts", "tests/test_api.py::test_user FAILED", "KEEP_PROVIDER_FAILURE", "Final error summary")
        MustNotContain = @("npm http fetch", "npm WARN deprecated", "Progress: resolved", "Downloaded crate_", "Compiling crate_", "[internal] load build definition", "[chromium]")
        Recommended = "ctxclean gha benchmarks/fixtures/provider_ci_mix.log --max-tokens 1600 --format markdown"
    },
    @{
        Name = "Stack trace dump"
        Fixture = "benchmarks/fixtures/stack_trace_dump_large.log"
        Args = @("--mode", "standard", "--max-tokens", "1850")
        MustContain = @("KEEP_STACK_ROOT", "UNIQUE_DATABASE_CONSTRAINT", "Collapsed stack frames")
        MustNotContain = @()
        Recommended = "ctxclean benchmarks/fixtures/stack_trace_dump_large.log --mode standard --max-tokens 1850"
    },
    @{
        Name = "Small dirty HTML"
        Fixture = "fixtures/dirty_html_article.html"
        Args = @("--mode", "standard")
        MustContain = @("API Setup & Troubleshooting", "Unique visible conclusion")
        MustNotContain = @("window.analytics", "Accept all cookies")
        Recommended = "ctxclean fixtures/dirty_html_article.html --mode standard"
    }
)

$results = [System.Collections.Generic.List[object]]::new()
foreach ($case in $cases) {
    $fixturePath = Join-Path $Root $case.Fixture
    $jsonText = & $ctxclean $fixturePath @($case.Args) --format json --quiet
    $parsed = $jsonText | ConvertFrom-Json
    $content = [string]$parsed.output.content
    $checks = [System.Collections.Generic.List[object]]::new()
    foreach ($needle in $case.MustContain) {
        $checks.Add([pscustomobject]@{
            type = "must_contain"
            value = $needle
            passed = $content.Contains($needle)
        })
    }
    foreach ($needle in $case.MustNotContain) {
        $checks.Add([pscustomobject]@{
            type = "must_not_contain"
            value = $needle
            passed = -not $content.Contains($needle)
        })
    }
    $failed = @($checks | Where-Object { -not $_.passed })
    if ($failed.Count -gt 0) {
        throw "Benchmark content check failed for $($case.Name)"
    }
    $results.Add([pscustomobject]@{
        name = $case.Name
        fixture = $case.Fixture
        command = "ctxclean $($case.Fixture) $($case.Args -join ' ') --format json --quiet"
        input_tokens = [int]$parsed.metrics.input_tokens
        output_tokens = [int]$parsed.metrics.output_tokens
        tokens_saved = [int]$parsed.metrics.tokens_saved
        reduction_percent = [double]$parsed.metrics.reduction_percent
        removed_section_kinds = @($parsed.removed_sections | ForEach-Object { $_.kind } | Sort-Object -Unique)
        warnings = @($parsed.warnings | ForEach-Object { $_.code })
        truncation_applied = [bool]$parsed.truncation.applied
        recommended_command = $case.Recommended
        checks = $checks
    })
}

$payload = [pscustomobject]@{
    generated_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    tokenizer = "o200k_base"
    binary = "target/release/ctxclean"
    results = $results
}
$json = ($payload | ConvertTo-Json -Depth 12) -replace "`r`n", "`n"
[System.IO.File]::WriteAllText($ResultsJson, $json + "`n", [System.Text.UTF8Encoding]::new($false))

$md = [System.Collections.Generic.List[string]]::new()
$md.Add("# ContextClean Benchmark Results")
$md.Add("")
$md.Add("Generated with ``scripts/benchmarks.ps1`` using the release ``ctxclean`` binary and exact ``o200k_base`` token counts.")
$md.Add("")
$md.Add("| Fixture | Input tokens | Output tokens | Tokens saved | Reduction | Recommended command |")
$md.Add("|---|---:|---:|---:|---:|---|")
foreach ($result in $results) {
    $md.Add("| $($result.name) | $($result.input_tokens) | $($result.output_tokens) | $($result.tokens_saved) | $([math]::Round($result.reduction_percent, 1))% | ``$($result.recommended_command)`` |")
}
$md.Add("")
$md.Add("All rows include must-keep and must-remove content checks in ``benchmarks/results.json``.")
Write-Utf8File -Path $ResultsMd -Lines $md

Write-Host "Wrote $ResultsJson"
Write-Host "Wrote $ResultsMd"
