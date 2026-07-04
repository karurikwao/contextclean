# Privacy

ContextClean V1 foundation is local-only.

## What Is Processed

The CLI reads files, directories, or stdin that the user provides.

## What Is Sent Over The Network

Nothing. The V1 foundation has no telemetry, analytics, remote error reporting, model API calls, or hosted sync.

## What Is Stored

ContextClean does not store persistent state. It only writes output when `--output` is provided.

## Sensitive Data

Directory scans skip common sensitive files and generated folders. Secret-like values are redacted by default before rendering output.
