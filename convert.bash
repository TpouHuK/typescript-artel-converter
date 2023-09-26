for f in $(find ./typescript_lib/ -type f -name "*.ts");
  do
  echo $f;
  y=$(echo ${f#./typescript_lib/});
  result1=${y#./typescript_lib/}
  result2="./typescript_lib_convert_test/${result1/.ts/.art}"
  ./target/debug/typescript-artel-converter $f >$result2
done
