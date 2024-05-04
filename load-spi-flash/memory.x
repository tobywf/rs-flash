/* SPDX-License-Identifier: MIT OR Apache-2.0 */
/* Memory map for GD32F307VE */
MEMORY
{
  /* Code area begins at 0x08000000 and has a size of 256kB */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  /* Data area begins at 0x08040000 and has a size of 256kB */
  /* ignored for simplicity */
  /* FLASH_RODATA : ORIGIN = ORIGIN(FLASH) + 256K, LENGTH = 256K */
  /* RAM begins at 0x20000000 and has a size of 96kB */
  RAM : ORIGIN = 0x20000000, LENGTH = 96K
}
