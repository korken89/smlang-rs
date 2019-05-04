set -euxo pipefail

main() {
    if [ $TARGET = docs ]; then
        mkdir mdcheck
        curl https://raw.githubusercontent.com/mike42/mdcheckr/master/mdcheckr -o mdcheck/mdcheckr
        chmod +x mdcheck/mdcheckr
    fi
}

main
