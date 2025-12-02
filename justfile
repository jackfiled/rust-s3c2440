build-app app:
    #!/usr/bin/env bash
    set -euxo pipefail
    cd applications/{{app}}
    cargo build

upload-app app: (build-app app)
    #!/usr/bin/env bash
    set -euxo pipefail
    scp target/armv4t-none-eabi/debug/{{app}} nas:/root/downloads/vxWorks.tq2440

copy-app app: (build-app app)
    #!/usr/bin/env bash
    set -euxo pipefail
    cp target/armv4t-none-eabi/debug/{{app}} tmp/vxWorks.tq2440
