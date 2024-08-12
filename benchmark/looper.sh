#! /bin/bash
source ./util.sh

mkdir result
copy_vmlinuz

for i in $(seq 1 $1); do
    ./run.sh
done

zip -r result/data.zip ./result/data
