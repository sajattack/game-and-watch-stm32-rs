MEMORY
{
  ITCMRAM  (xrw) : ORIGIN = 0x00000000, LENGTH = 64K
  DTCMRAM  (xrw) : ORIGIN = 0x20000000, LENGTH = 128K
  RAM      (xrw) : ORIGIN = 0x24000000, LENGTH = 1024K
  FLASH    (xr ) : ORIGIN = 0x8000000,  LENGTH = 128K
  EXTFLASH (xr ) : ORIGIN = 0x90000000, LENGTH = 1024K
}

SECTIONS
{
  ._extflash :
  {
    . = ALIGN(4);
    _extflash = .;       /* define a global symbols to point at the external flash */
    KEEP(*(._extflash))
  } >EXTFLASH 
}
