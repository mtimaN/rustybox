#!/bin/bash

node -e 'require("lodash");' &> /dev/null

if [ $? != 0 ]; then
    echo "Packages is not installed, installing"
    npm install install --production 
fi

export QUOTING_STYLE=literal

function run_script
{
    name=`basename $1`
    chmod u+x $1
    # in output/$name.out se va stoca outputul rularii scriptului python
    timeout 5 "./$1"  "results/$name.out" "results/$name.test" >> errors 2>&1
    result=$?
    echo Test $name >> "$errorslist" 2>&1
    cat "results/$name.out" >> "$errorslist" 2>> errors
    echo "" >> "$errorslist" 2>> errors
    if [ $result != 0 ]
    then
    # daca nu a facut bine, copiem ce a afisat scriptul la rulare in error.out
        echo >> "$errorslist" 2>&1
        cat "results/$name.test"  > "$hintsfile" 2>> errors
    fi
    rm -f "results/$name.test" >> errors 2>&1
    rm -f "results/$name.out" >> errors 2>&1
    return $result
}

DIR=`dirname $0`

passed=0
failed=0
total=0

POINTS=0
POINTS_TOTAL=0

cd "$DIR"

rm -rf "$DIR/output"
mkdir "$DIR/output"

rm -rf "$DIR/results"
mkdir "$DIR/results"

errorslist=$DIR/results/errors.out
hintsfile=$DIR/results/hints.out
rm -f $errorslist
rm -f $hintsfile

if [ $# -lt 1 ];
then
    echo "Running all tests"
    for folder in $(cd "$DIR" && find verify -mindepth 1 -maxdepth 1 -type d)
    do
        for script in "$folder"/*.sh
        do
            P=`head -n 2 "$script" | tail -n 1 | cut -d ' ' -f 2`
            # echo "cd devoir-1-tests && ./run_all.sh $script" "$P"
            # continue
            
            POINTS_TOTAL=$(($POINTS_TOTAL+$P))
            
            title=`basename $script`
            strtitle="Verifying $title"
            printf '%s' "$strtitle"
            pad=$(printf '%0.1s' "."{1..60})
            padlength=65

            # P=`head -n 2 "$script" | tail -n 1 | cut -d ' ' -f 2`

            

            if run_script $script
            then
                str="ok (""$P""p/""$P""p)"
                passed=$(($passed+1))
                POINTS=$(($POINTS+$P))
            else
                if [ $? == 124 ];
                then
                    str="timeout (0p/""$P""p)"
                else
                    str="error (0p/""$P""p)"
                fi
                failed=$(($failed+1))                
            fi
            total=$(($total+1))
            printf '%*.*s' 0 $((padlength - ${#strtitle} - ${#str} )) "$pad"
            printf '%s\n' "$str"
            if [ -f "$hintsfile" ]
            then
                cat "$hintsfile" 2>> errors
            fi
            rm -f "$hintsfile" 2>> errors
        done
    done

    echo 'Tests: ' $passed '/' $total
    echo 'Points: '$POINTS '/' $POINTS_TOTAL
    echo 'Mark without penalties: '`echo $(($POINTS/6)) | sed 's/.$/.&/'`

    node index.js "$POINTS/$POINTS_TOTAL"

    # afisam problemele
    echo
    cat $errorslist 2>> errors
    echo

    if [ $passed < $total ];
    then
        exit -1
    fi
else
    rm -rf $errorslist
    rm -rf $hintsfile
    echo "Run int folder $(pwd)"
    run_script "verify/$1"
    error=$?
    if [ $error != 0 ]
    then
        echo "Output"
        echo "------"
        cat $errorslist
        if [ -f $hintsfile ]
        then
            echo "Hints"
            echo "-----"
            cat $hintsfile
        fi
    fi
    exit $error
fi




