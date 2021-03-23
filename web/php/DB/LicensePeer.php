<?php

namespace Wikidot\DB;








/**
 * Object Model Class.
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
