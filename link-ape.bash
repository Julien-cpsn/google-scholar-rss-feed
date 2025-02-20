# look at the built debug binaries
#ls ./target/x86_64-unknown-linux-cosmo/debug/*.com.dbg

# apelink
MODE=
APELINK=$COSMO/o/tool/build/apelink
apelinkpls () {
    OUTPUT="$1"
    OUTPUT_X86_64="$2"
    "$APELINK" -l "$COSMO/o/$MODE/ape/ape.elf" \
        -o "$OUTPUT" \
        "$OUTPUT_X86_64"
}

apelinkpls ./rust-cosmopolitan.com ./target/x86_64-unknown-linux-cosmo/debug/rust-cosmopolitan.com.dbg
