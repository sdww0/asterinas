#! /bin/bash
source ./util.sh
mkdir result/data
mkdir result/fig

# Get data process files
dataProcess=$(iterDir "data")
echo "Running data processor:"
for file in ${dataProcess[@]}; do
    if [[ $(is_excluded $file) == 1 ]]; then
        continue
    else
        echo "$file"
    fi
done

# Run data processor
for file in ${dataProcess[@]}; do
    if [[ $(is_excluded $file) == 1 ]]; then
        continue
    else
        python $file
    fi
done
