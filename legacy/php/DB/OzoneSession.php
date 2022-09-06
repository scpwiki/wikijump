<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class OzoneSession extends \Ozone\Framework\DB\OzoneSession
{
    /**
     *  Look man. I know how dumb this looks. You're not telling me anything I don't know.
     *  But if you remove this class, or rename it, you'll break logon sessions.
     *  Blame Ozone\Framework\Database\BaseDBPeer.
     */
}
