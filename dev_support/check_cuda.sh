#!/bin/bash

cd $(dirname "$0")

# Detect if cuda installed
if command -v nvcc >/dev/null 2>&1; then
    # 创建一个临时 CUDA 文件
    cat <<EOF > test_cuda.cu
    #include <stdio.h>
    
    __global__ void helloFromGPU() {
        printf("CUDA test success!\\n");
    }

    int main() {
        helloFromGPU<<<1, 1>>>();
        cudaDeviceSynchronize();
        return 0;
    }
EOF

    nvcc -o test_cuda test_cuda.cu && ./test_cuda
    if [ $? -eq 0 ]; then
        rm -f test_cuda.cu test_cuda
        exit 0
    else
        rm -f test_cuda.cu test_cuda
        echo "Cannot compile and run CUDA source code"
        exit 1
    fi
else
    echo "CUDA Enviroment not installed"
    exit 1
fi

