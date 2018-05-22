# tixml2svd

This utility creates
[SVD](https://www.keil.com/pack/doc/CMSIS/SVD/html/svd_Format_pg.html)
files from the Texas-Instruments XML (called TIXML from now on) device
and peripheral descriptor files.

Device descriptor files are generally found in the
ccsv8/ccs_base/common/targetdb/devices directory of a TI Code Composer
installation directory. They contain the names and base addresses of
all of the device's peripherals, as well as a the relative path of
each peripheral's descriptor file.

Peripheral descriptor files are generally found in the
ccsv8/ccs_base/common/targetdb/Modules directory of a TI Code Composer
installation directory. They contain the names and addresses of all of
the registers belonging to a peripheral.

## Usage

Note: you must first remove any byte-order-mark (BOM) from your device
file. This is a sequence of invisible bytes that appears at the
beginning of certain text files. One way to do this is with the GNU
sed commande, issued from a bash prompt:

`sed $'1s/^\uFEFF//' < device.xml > device_wo_bom.xml`

Now, process your device file with something like `tixml2svd -i
cc2652r1f.xml`. If this does not work, try one of the device
peripherals all by itself, with something like `tixml2svd -p -i
Modules/CC26xx/CC2652/PRCM.xml`.

## Caveats

I have tried this code on the CC2652. It detected a bug in the
CRYPTO.xml file (a register with a width of 33 bits). After fixing the
original TIXML file, it works fine.

For the moment, this utility does not generate SVD device
headers. This will require a little research on your part. This is an
example of the information you will need to dig up (the following
header provides sufficient information for the Segger Ozone debugger).

```
<?xml version="1.0" encoding="UTF-8"?>
<device xmlns:xs="http://www.w3.org/2001/XMLSchema-instance" schemaVersion="1.1" xs:noNamespaceSchemaLocation="CMSIS-SVD_Schema_1_0.xsd">
  <name>CC26x0</name>
  <version>2.3</version>
  <description>SimpleLink CC26xx Ultra-low power wireless MCU</description>
  <cpu>
    <name>CM3</name>
    <revision>r2p1</revision>
    <endian>little</endian>
    <mpuPresent>false</mpuPresent>
    <fpuPresent>false</fpuPresent>
    <nvicPrioBits>3</nvicPrioBits>
    <vendorSystickConfig>false</vendorSystickConfig>
  </cpu>
  <addressUnitBits>8</addressUnitBits>
  <width>32</width>
  <size>32</size>
  <access>read-write</access>
  <resetMask>0xFFFFFFFF</resetMask>
```
