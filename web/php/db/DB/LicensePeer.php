<?php


namespace DB;





/**
 * Object Model class.
 *
 */
class LicensePeer extends LicensePeerBase
{

    public function selectById($licenseId)
    {

        $license = $this->selectByPrimaryKey($licenseId);
        return $license;
    }
}
