<?php
class FlickrHandler extends phpFlickr
{

     public $cache = true;

    private static $instance;

    public static function instance()
    {
        if (self::$instance == null) {
            // get the flickr key
            $key = GlobalProperties::$FLICKR_API_KEY;
            self::$instance = new FlickrHandler($key, null, false);
        }
        return self::$instance;
    }

    function enableCache($type, $connection, $cache_expire = 600, $table = 'flickr_cache')
    {
    }

    function getCached($request)
    {
        $reqhash = md5(serialize($request));
        $key = "phpflickrcache..".$reqhash;
        $mc = Ozone::$memcache;
        $out = $mc->get($key);
        if ($out != false) {
            return $out;
        }
        return false;
    }

    public function cache($request, $response)
    {
        $reqhash = md5(serialize($request));
        $key = "phpflickrcache..".$reqhash;
        $mc = Ozone::$memcache;
        $mc->set($key, $response, 0, 600);
        return false;
    }
}
