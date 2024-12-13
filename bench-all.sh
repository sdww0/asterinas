
function iterDir() {
    files=()
    for file in $(ls $1); do
        if [ -d $1"/"$file ]; then
            files+=($(iterDir $1"/"$file))
        else
            files+=($1"/"$file)
        fi
    done
    for file in ${files[@]}; do
        echo $file
    done
}

BASE_DIR="test/benchmark/lmbench"

benchs=$(ls $BASE_DIR)

echo "Running benchs:"
for file in ${benchs[@]}; do
    if [ -f $BASE_DIR"/"$file ]; then
        continue
    fi
    benchmark_dir=$BASE_DIR"/"$file
    files=$(ls $benchmark_dir)
    is_host=0
    
    for temp_file in ${files[@]}; do
        if [[ $temp_file == *"host"* ]]; then
            is_host=1
        fi
    done

    echo "${benchmark_dir:15}"
    
    if [ $is_host -eq 1 ]; then
        echo "Running host_guest"
        $(bash ./test/benchmark/bench_linux_and_aster.sh "${benchmark_dir:15}" host_guest)
    else
        echo "Running guest_only"
        $(bash ./test/benchmark/bench_linux_and_aster.sh "${benchmark_dir:15}")
    fi

done



