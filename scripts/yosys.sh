#!/bin/bash

yosys -p "
    read_verilog -sv circuits/adder.v

    hierarchy -check -auto-top;
    proc; memory; fsm; wreduce; opt -full
    techmap; opt -full

    rmports
    splitnets -ports
    clean -purge
    show -stretch -format ps -viewer evince
    write_json design.json"
