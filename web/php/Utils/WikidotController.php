<?php

namespace Wikidot\Utils;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Ozone\Framework\WebFlowController;

use Wikidot\DB\Site;
use Wikidot\DB\SitePeer;
use Wikidot\DB\MemberPeer;
use Wikijump\Models\User;

abstract class WikidotController extends WebFlowController
{

    protected static $HTML_MIME_TYPES = ";^text/html|^application/xhtml+xml|^application/xml|^text/xml;";
    protected static $HTML_SERVE_AS = "text/plain";

    /**
     * Gets a site from given hostname. This version works for custom domains and upload domain if needed
     *
     * @param string $siteHost the host to check
     * @param bool $customDomains whether to check custom domains
     * @param bool $uploadDomain whether to check upload domains as well
     * @return Site
     */
    protected function siteFromHost($siteHost, $customDomains = false, $uploadDomain = false)
    {
        if ($uploadDomain) {
            $regexp = "/^([a-zA-Z0-9\-]+)\.(" . GlobalProperties::$URL_DOMAIN_PREG . "|" . GlobalProperties::$URL_UPLOAD_DOMAIN_PREG . ")$/";
        } else {
            $regexp = "/^([a-zA-Z0-9\-]+)\.(" . GlobalProperties::$URL_DOMAIN_PREG . ")$/";
        }

        if (preg_match($regexp, $siteHost, $matches) == 1) {
            // select site based on the unix name

            $siteUnixName = $matches[1];
            $mcKey = 'site..'.$siteUnixName;
            $site = Cache::get($mcKey);
            if ($site == false) {
                $c = new Criteria();
                $c->add("unix_name", $siteUnixName);
                $c->add("site.deleted", false);
                $site = SitePeer::instance()->selectOne($c);
                if ($site) {
                    Cache::put($mcKey, $site, 864000);
                }
            }
        }


        // select site based on the custom domain

        if (! $site && $customDomains) {
            $mcKey = 'site_cd..'.$siteHost;
            $site = Cache::get($mcKey);
            if ($site == false) {
                $c = new Criteria();
                $c->add("custom_domain", $siteHost);
                $c->add("site.deleted", false);
                $site = SitePeer::instance()->selectOne($c);
                if ($site) {
                    Cache::put($mcKey, $site, 3600);
                }
            }
        }

        return $site;
    }

    protected function isUploadDomain($siteHost)
    {

        if (preg_match("/^[^.]*\." . GlobalProperties::$URL_UPLOAD_DOMAIN_PREG . "$/", $siteHost)) {
            return true;
        }

        return false;
    }

    protected function siteNotExists()
    {
        $this->serveFile(WIKIJUMP_ROOT."/resources/views/site_not_exists.html", "text/html");
    }

    protected function isBuggyIeDamnYouBastard()
    {
        if (isset($_SERVER['HTTP_USER_AGENT']) && strpos($_SERVER['HTTP_USER_AGENT'], 'MSIE') !== false) {
            return true;
        } else {
            return false;
        }
    }

    protected function fileNotExists()
    {
        $this->serveFile(WIKIJUMP_ROOT."/resources/views/file_not_exists.html", "text/html");
    }

    private function calculateEtag($path)
    {
        if (file_exists($path)) {
            $stat = stat($path);
            if ($stat) {
                return '"' . md5($stat['mtime']) . '"';
            }
        }
        return '"none"';
    }

    public function return304()
    {
        header("HTTP/1.0 304 Not Modified");
    }

    /**
     * serves a file of given path with autodetected MIME type and given expires (if any)
     *
     * @param string $path
     * @param int $expires time in seconds
     */
    protected function serveFileWithMime($path, $expires = null, $restrictHtml = false)
    {
        $etag = $this->calculateEtag($path);

        if (isset($_SERVER["HTTP_IF_NONE_MATCH"])) {
            if ($_SERVER["HTTP_IF_NONE_MATCH"] == $etag) {
                $this->return304();
                return;
            }
        }

        /* guess/set the mime type for the file */
        if ($path == "theme" || preg_match("/\.css$/", $path)) {
            $mime = "text/css";
        } elseif (preg_match("/\.js$/", $path)) {
            $mime = "text/javascript";
        }

        if (! isset($mime)) {
            $mime = $this->fileMime($path, $restrictHtml);
        }

        $this->serveFile($path, $mime, $expires, $etag);
    }

    /**
     * checks if the user is a member of a site
     *
     * @param User $user
     * @param Site $site
     * @return boolean
     */
    protected function member($user, $site)
    {
        if (! $site || ! $user) {
            return false;
        }

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $user->id);

        if (MemberPeer::instance()->selectOne($c)) { // user is a member of the Wiki
            return true;
        }

        return false;
    }

    /**
     * detects MIME type of a file. Includes workarounds for buggy detection
     *
     * @param string $path path to file
     * @return string the MIME type
     */
    protected function fileMime($path, $restrictHtml = false)
    {

        if (file_exists($path)) {
            $mime =  FileMime::mime($path);
        } else {
            $mime = false;
        }

        if (! $mime || $mime == "application/msword") {
            $mime = "application/octet-stream";
        }

        if ($restrictHtml && preg_match(self::$HTML_MIME_TYPES, $mime)) {
            $mime = self::$HTML_SERVE_AS;
        }

        return $mime;
    }
}
