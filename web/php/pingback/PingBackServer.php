<?php
/* TODO: static? */
class PingBackServer
{
    /**
     * ping method for pingback xmlrpc server
     *
     * @param string $sourceURI The absolute URI of the post on the source page containing the link to the target site.
     * @param string $targetURI The absolute URI of the target of the link, as given on the source page.
     * @throws Zend_XmlRpc_Server_Fault on caugth exceptions
     * @return string on success
     */
    public static function ping($sourceURI, $targetURI)
    {

        $pb = new PingBack($sourceURI, $targetURI);

        Zend_XmlRpc_Server_Fault::attachFaultException("PingBackException");

        $ret = $pb->pong();

        // do something

        return "OK";
    }
}
