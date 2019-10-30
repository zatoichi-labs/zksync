#!/bin/bash

. .setup_env

FRANKLIN_HOME=`dirname $0`/..

jq '{ abi: .abi, interface: .interface }' $FRANKLIN_HOME/contracts/build/Franklin.json > $FRANKLIN_HOME/src/js/franklin_lib/abi/Franklin.json
