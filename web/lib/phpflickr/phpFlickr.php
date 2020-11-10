<?php
/* phpFlickr Class 1.6-beta
 * Written by Dan Coulter (dan@dancoulter.com)
 * Sourceforge Project Page: http://www.sourceforge.net/projects/phpflickr/
 * Released under GNU General Public License (http://www.gnu.org/copyleft/gpl.html)
 * For more information about the class and upcoming tools and toys using it,
 * visit http://www.phpflickr.com/ or http://phpflickr.sourceforge.net
 *
 *     For installation instructions, open the README.txt file packaged with this
 *     class. If you don't have a copy, you can see it at:
 *     http://www.phpflickr.com/README.txt
 *
 *     Please submit all problems or questions to the Help Forum on my project page:
 *         http://sourceforge.net/forum/forum.php?forum_id=469652
 *
 */
if (session_id() == "") {
    session_start();
}
require_once('xml.php');

// Decides which include path delimiter to use.  Windows should be using a semi-colon
// and everything else should be using a colon.  If this isn't working on your system,
// comment out this if statement and manually set the correct value into $path_delimiter.
if (strpos(__FILE__, ':') !== false) {
    $path_delimiter = ';';
} else {
    $path_delimiter = ':';
}

// This will add the packaged PEAR files into the include path for PHP, allowing you
// to use them transparently.  This will prefer officially installed PEAR files if you
// have them.  If you want to prefer the packaged files (there shouldn't be any reason
// to), swap the two elements around the $path_delimiter variable.  If you don't have
// the PEAR packages installed, you can leave this like it is and move on.

ini_set('include_path', ini_get('include_path') . $path_delimiter . dirname(__FILE__) . '/PEAR');

// If you have problems including the default PEAR install (like if your open_basedir
// setting doesn't allow you to include files outside of your web root), comment out
// the line above and uncomment the next line:

// ini_set('include_path', dirname(__FILE__) . '/PEAR' . $path_delimiter . ini_get('include_path'));

class phpFlickr {
    public $api_key;
    public $secret;
    public $REST = 'http://www.flickr.com/services/rest/';
    public $Upload = 'http://www.flickr.com/services/upload/';
    public $Replace = 'http://www.flickr.com/services/replace/';
    public $xml_parser;
    public $req;
    public $response;
    public $parsed_response;
    public $cache = false;
    public $cache_db = null;
    public $cache_table = null;
    public $cache_dir = null;
    public $cache_expire = null;
    public $die_on_error;
    public $error_code;
    public $error_msg;
    public $token;
    public $php_version;
    public $service;

    function phpFlickr ($api_key, $secret = NULL, $die_on_error = true)
    {
        //The API Key must be set before any calls can be made.  You can
        //get your own at http://www.flickr.com/services/api/misc.api_keys.html
        $this->api_key = $api_key;
        $this->secret = $secret;
        $this->die_on_error = $die_on_error;
        $this->service = "flickr";

        //Find the PHP version and store it for future reference
        $this->php_version = explode("-", phpversion());
        $this->php_version = explode(".", $this->php_version[0]);

        //All calls to the API are done via the POST method using the PEAR::HTTP_Request package.
        require_once 'HTTP/Request.php';
        $this->req = new HTTP_Request();
        $this->req->setMethod(HTTP_REQUEST_METHOD_POST);

        //setup XML parser using Aaron Colflesh's XML class.
        $this->xml_parser = new xml(false, true, true);
    }

    function enableCache($type, $connection, $cache_expire = 600, $table = 'flickr_cache')
    {
        // Turns on caching.  $type must be either "db" (for database caching) or "fs" (for filesystem).
        // When using db, $connection must be a PEAR::DB connection string. Example:
        //      "mysql://user:password@server/database"
        // If the $table, doesn't exist, it will attempt to create it.
        // When using file system, caching, the $connection is the folder that the web server has write
        // access to. Use absolute paths for best results.  Relative paths may have unexpected behavior
        // when you include this.  They'll usually work, you'll just want to test them.
        if ($type == 'db') {
            require_once 'DB.php';
            $db =& DB::connect($connection);
            if (PEAR::isError($db)) {
                die($db->getMessage());
            }

            $db->query("
                CREATE TABLE IF NOT EXISTS `$table` (
                    `request` CHAR( 35 ) NOT NULL ,
                    `response` TEXT NOT NULL ,
                    `expiration` DATETIME NOT NULL ,
                    INDEX ( `request` )
                ) TYPE = MYISAM");
            $db->query("DELETE FROM $table WHERE expiration < DATE_SUB(NOW(), INTERVAL $cache_expire second)");
            if (strpos($connection, 'mysql') !== false) {
                $db->query('OPTIMIZE TABLE ' . $table);
            }
            $this->cache = 'db';
            $this->cache_db = $db;
            $this->cache_table = $table;
        } elseif ($type = 'fs') {
            $this->cache = 'fs';
            $connection = realpath($connection);
            $this->cache_dir = $connection;
            if ($dir = opendir($this->cache_dir)) {
                while ($file = readdir($dir)) {
                    if (substr($file, -6) == '.cache' && ((filemtime($this->cache_dir . '/' . $file) + $cache_expire) < time()) ) {
                        unlink($this->cache_dir . '/' . $file);
                    }
                }
            }
        }
        $this->cache_expire = $cache_expire;
    }

    function getCached ($request)
    {
        //Checks the database or filesystem for a cached result to the request.
        //If there is no cache result, it returns a value of false. If it finds one,
        //it returns the unparsed XML.
        $reqhash = md5(serialize($request));
        if ($this->cache == 'db') {
            $result = $this->cache_db->getOne("SELECT response FROM " . $this->cache_table . " WHERE request = '" . $reqhash . "'");
            if (!empty($result)) {
                return $result;
            }
        } elseif ($this->cache == 'fs') {
            $file = $this->cache_dir . '/' . $reqhash . '.cache';
            if (file_exists($file)) {
				if ($this->php_version[0] > 4 || ($this->php_version[0] == 4 && $this->php_version[1] >= 3)) {
					return file_get_contents($file);
				} else {
					return implode('', file($file));
				}
            }
        }
        return false;
    }

    function cache ($request, $response)
    {
        //Caches the unparsed XML of a request.
        $reqhash = md5(serialize($request));
        if ($this->cache == 'db') {
            $this->cache_db->query("DELETE FROM $this->cache_table WHERE request = '$reqhash'");
            $sql = "INSERT INTO " . $this->cache_table . " (request, response, expiration) VALUES ('$reqhash', '" . str_replace("'", "''", $response) . "', '" . strftime("%Y-%m-%d %H:%M:%S") . "')";
            $this->cache_db->query($sql);
        } elseif ($this->cache == "fs") {
            $file = $this->cache_dir . "/" . $reqhash . ".cache";
            $fstream = fopen($file, "w");
            $result = fwrite($fstream,$response);
            fclose($fstream);
            return $result;
        }
        return false;
    }

    function request ($command, $args = array(), $nocache = false)
    {
        //Sends a request to Flickr's REST endpoint via POST.
        $this->req->setURL($this->REST);
        $this->req->clearPostData();
        if (substr($command,0,7) != "flickr.") {
            $command = "flickr." . $command;
        }

        //Process arguments, including method and login data.
        $args = array_merge(array("method" => $command, "api_key" => $this->api_key), $args);
        if (!empty($this->token)) {
            $args = array_merge($args, array("auth_token" => $this->token));
        } elseif (!empty($_SESSION['phpFlickr_auth_token'])) {
            $args = array_merge($args, array("auth_token" => $_SESSION['phpFlickr_auth_token']));
        }
        ksort($args);
        $auth_sig = "";
        if (!($this->response = $this->getCached($args)) || $nocache) {
            foreach ($args as $key => $data) {
                $auth_sig .= $key . $data;
                $this->req->addPostData($key, $data);
            }
            if (!empty($this->secret)) {
                $api_sig = md5($this->secret . $auth_sig);
                $this->req->addPostData("api_sig", $api_sig);
            }

            //Send Requests
            if ($this->req->sendRequest()) {
                $this->response = $this->req->getResponseBody();
                $this->cache($args, $this->response);
            } else {
                die("There has been a problem sending your command to the server.");
            }
        }
        return $this->response;
    }

    function parse_response ($xml = NULL)
    {
        //Sends response data through XML parser and returns an associative array.
        if ($xml === NULL) {
            $xml = $this->response;
        }
        $this->parsed_response = $this->xml_parser->parse($xml);

        //Check for an error and die if it finds one.
        if (!empty($this->parsed_response['rsp']['err']) && $this->die_on_error) {
            die("The Flickr API returned error code #" . $this->parsed_response['rsp']['err']['code'] . ": " . $this->parsed_response['rsp']['err']['msg']);
        } elseif (!empty($this->parsed_response['rsp']['err'])) {
			$this->error_code = $this->parsed_response['rsp']['err']['code'];
			$this->error_msg = "The Flickr API returned error code #" . $this->parsed_response['rsp']['err']['code'] . ": " . $this->parsed_response['rsp']['err']['msg'];
			return false;
        } else {
			$this->error_code = false;
			$this->error_msg = false;
        }

        return $this->parsed_response['rsp'];
    }

    function setService($service)
    {
		// Sets which service to connect to.  Currently supported services are
		// "flickr" and "23"
		if ($service == "23") {
			$this->service = "23";
			$this->REST = 'http://www.23hq.com/services/rest/';
			$this->Upload = 'http://www.23hq.com/services/upload/';
			$this->Replace = 'http://www.23hq.com/services/replace/';
		} elseif (strtolower($service) == "flickr") {
			$this->service = "flickr";
			$this->REST = 'http://www.flickr.com/services/rest/';
			$this->Upload = 'http://www.flickr.com/services/upload/';
			$this->Replace = 'http://www.flickr.com/services/replace/';
		} else {
			die ("You have entered a service that does not exist or is not supported at this time.");
		}
    }

    function setToken($token)
    {
        // Sets an authentication token to use instead of the session variable
        $this->token = $token;
    }

    function setProxy($server, $port)
    {
        // Sets the proxy for all phpFlickr calls.
        $this->req->setProxy($server, $port);
    }

    function getErrorCode()
    {
		// Returns the error code of the last call.  If the last call did not
		// return an error. This will return a false boolean.
		return $this->error_code;
    }

    function getErrorMsg()
    {
		// Returns the error message of the last call.  If the last call did not
		// return an error. This will return a false boolean.
		return $this->error_msg;
    }

    /* These functions are front ends for the flickr calls */

    function buildPhotoURL ($photo, $size = "Medium")
    {
        //var_dump($photo);
        //receives an array (can use the individual photo data returned
        //from an API call) and returns a URL (doesn't mean that the
        //file size exists)
		if ($this->service == "23") {
			$url = "http://www.23hq.com/";
		} else {
			$url = "http://farm".$photo['farm'].".static.flickr.com/";
        }
        if(strtolower($size) == 'original') {
            $url .= $photo['server'] . "/" . $photo['id'] . "_" . $photo['originalsecret'] . "_o" . "." . $photo['originalformat'];
        } else {
            $url .= $photo['server'] . "/" . $photo['id'] . "_" . $photo['secret'];
            switch (strtolower($size)) {
                case "square":
                    $url .= "_s";
                    break;
                case "thumbnail":
                    $url .= "_t";
                    break;
                case "small":
                    $url .= "_m";
                    break;
                case "medium":
                    $url .= "";
                    break;
                case "large":
                    $url .= "_b";
                    break;

            }
            $url .= ".jpg";
        }
        return $url;
    }

    function sync_upload ($photo, $title = null, $description = null, $tags = null, $is_public = null, $is_friend = null, $is_family = null) {
        $this->req->setURL($this->Upload);
        $this->req->clearPostData();

        //Process arguments, including method and login data.
        $args = array("api_key" => $this->api_key, "title" => $title, "description" => $description, "tags" => $tags, "is_public" => $is_public, "is_friend" => $is_friend, "is_family" => $is_family);
        if (!empty($this->email)) {
            $args = array_merge($args, array("email" => $this->email));
        }
        if (!empty($this->password)) {
            $args = array_merge($args, array("password" => $this->password));
        }
        if (!empty($this->token)) {
            $args = array_merge($args, array("auth_token" => $this->token));
        } elseif (!empty($_SESSION['phpFlickr_auth_token'])) {
            $args = array_merge($args, array("auth_token" => $_SESSION['phpFlickr_auth_token']));
        }

        ksort($args);
        $auth_sig = "";
        foreach ($args as $key => $data) {
            if ($data !== null) {
                $auth_sig .= $key . $data;
                $this->req->addPostData($key, $data);
            }
        }
        if (!empty($this->secret)) {
            $api_sig = md5($this->secret . $auth_sig);
            $this->req->addPostData("api_sig", $api_sig);
        }

        $photo = realpath($photo);

        $result = $this->req->addFile("photo", $photo);

        if (PEAR::isError($result)) {
            die($result->getMessage());
        }

        //Send Requests
        if ($this->req->sendRequest()) {
            $this->response = $this->req->getResponseBody();
        } else {
            die("There has been a problem sending your command to the server.");
        }
        $result = $this->parse_response();
        return $result['photoid'];
    }

    function async_upload ($photo, $title = null, $description = null, $tags = null, $is_public = null, $is_friend = null, $is_family = null) {
        $this->req->setURL($this->Upload);
        $this->req->clearPostData();

        //Process arguments, including method and login data.
        $args = array("async" => 1, "api_key" => $this->api_key, "title" => $title, "description" => $description, "tags" => $tags, "is_public" => $is_public, "is_friend" => $is_friend, "is_family" => $is_family);
        if (!empty($this->email)) {
            $args = array_merge($args, array("email" => $this->email));
        }
        if (!empty($this->password)) {
            $args = array_merge($args, array("password" => $this->password));
        }
        if (!empty($this->token)) {
            $args = array_merge($args, array("auth_token" => $this->token));
        } elseif (!empty($_SESSION['phpFlickr_auth_token'])) {
            $args = array_merge($args, array("auth_token" => $_SESSION['phpFlickr_auth_token']));
        }

        ksort($args);
        $auth_sig = "";
        foreach ($args as $key => $data) {
            if ($data !== null) {
                $auth_sig .= $key . $data;
                $this->req->addPostData($key, $data);
            }
        }
        if (!empty($this->secret)) {
            $api_sig = md5($this->secret . $auth_sig);
            $this->req->addPostData("api_sig", $api_sig);
        }

        $photo = realpath($photo);

        $result = $this->req->addFile("photo", $photo);

        if (PEAR::isError($result)) {
            die($result->getMessage());
        }

        //Send Requests
        if ($this->req->sendRequest()) {
            $this->response = $this->req->getResponseBody();
        } else {
            die("There has been a problem sending your command to the server.");
        }
        $result = $this->parse_response();
        return $result['ticketid'];
    }

    // Interface for new replace API method.
    function replace ($photo, $photo_id, $async = null) {
        $this->req->setURL($this->Replace);
        $this->req->clearPostData();

        //Process arguments, including method and login data.
        $args = array("api_key" => $this->api_key, "photo_id" => $photo_id, "async" => $async);
        if (!empty($this->email)) {
            $args = array_merge($args, array("email" => $this->email));
        }
        if (!empty($this->password)) {
            $args = array_merge($args, array("password" => $this->password));
        }
        if (!empty($this->token)) {
            $args = array_merge($args, array("auth_token" => $this->token));
        } elseif (!empty($_SESSION['phpFlickr_auth_token'])) {
            $args = array_merge($args, array("auth_token" => $_SESSION['phpFlickr_auth_token']));
        }

        ksort($args);
        $auth_sig = "";
        foreach ($args as $key => $data) {
            if ($data !== null) {
                $auth_sig .= $key . $data;
                $this->req->addPostData($key, $data);
            }
        }
        if (!empty($this->secret)) {
            $api_sig = md5($this->secret . $auth_sig);
            $this->req->addPostData("api_sig", $api_sig);
        }

        $photo = realpath($photo);

        $result = $this->req->addFile("photo", $photo);

        if (PEAR::isError($result)) {
            die($result->getMessage());
        }

        //Send Requests
        if ($this->req->sendRequest()) {
            $this->response = $this->req->getResponseBody();
        } else {
            die("There has been a problem sending your command to the server.");
        }
        $result = $this->parse_response();
        return $result['photoid'];
    }

    function auth ($perms = "read", $remember_uri = true)
    {
        // Redirects to Flickr's authentication piece if there is no valid token.
        // If remember_uri is set to false, the callback script (included) will
        // redirect to its default page.

        if (empty($_SESSION['phpFlickr_auth_token']) && empty($this->token)) {
            if ($remember_uri) {
                $redirect = $_SERVER['REQUEST_URI'];
            }
            $api_sig = md5($this->secret . "api_key" . $this->api_key . "extra" . $redirect . "perms" . $perms);
			if ($this->service == "23") {
				header("Location: http://www.23hq.com/services/auth/?api_key=" . $this->api_key . "&extra=" . $redirect . "&perms=" . $perms . "&api_sig=". $api_sig);
			} else {
				header("Location: http://www.flickr.com/services/auth/?api_key=" . $this->api_key . "&extra=" . $redirect . "&perms=" . $perms . "&api_sig=". $api_sig);
			}
            exit;
        } else {
            $tmp = $this->die_on_error;
            $this->die_on_error = false;
            $rsp = $this->auth_checkToken();
            if ($this->error_code !== false) {
                unset($_SESSION['phpFlickr_auth_token']);
                $this->auth($perms, $remember_uri);
            }
            $this->die_on_error = $tmp;
            return $rsp['perms'];
        }
    }

    /*******************************

    To use the phpFlickr::call method, pass a string containing the API method you want
    to use and an associative array of arguments.  For example:
        $result = $f->call("flickr.photos.comments.getList", array("photo_id"=>'34952612'));
    This method will allow you to make calls to arbitrary methods that haven't been
    implemented in phpFlickr yet.  This will be especially useful if 23 expands their
    API beyond the Flickr functionality and I don't expand my support.  You need to
    be careful though; if you call a method that returns a list of items (tags on a photo
    or comments, for example) and there is only one item in the list, the XML parser won't
    return the element as an array.  You'll need to code that in after you get the result
    back.  One example of this is in blogs_getList().

    *******************************/

    function call($method, $arguments)
    {
        $this->request($method, $arguments);
        return $this->parse_response();
    }

    /*
        These functions are the direct implementations of flickr calls.
        For method documentation, including arguments, visit the address
        included in a comment in the function.
    */

    /* Authentication methods */
    function auth_checkToken ()
    {
        /* http://www.flickr.com/services/api/flickr.auth.checkToken.html */
        $this->request('flickr.auth.checkToken');
        $result = $this->parse_response();
        return $result['auth'];
    }

    function auth_getFrob ()
    {
        /* http://www.flickr.com/services/api/flickr.auth.getFrob.html */
        $this->request('flickr.auth.getFrob');
        $result = $this->parse_response();
        return $result['frob'];
    }

    function auth_getToken ($frob)
    {
        /* http://www.flickr.com/services/api/flickr.auth.getToken.html */
        $this->request('flickr.auth.getToken', array('frob'=>$frob));
        $result = $this->parse_response();
        session_register('phpFlickr_auth_token');
        $_SESSION['phpFlickr_auth_token'] = $result['auth']['token'];
        return $result['auth']['token'];
    }

/* Blogs methods */
    function blogs_getList ()
    {
        /* http://www.flickr.com/services/api/flickr.blogs.getList.html */
        $this->request('flickr.blogs.getList');
        $this->parse_response();
        $result = $this->parsed_response['rsp']['blogs'];
        if (!empty($result['blog']['id'])) {
            $tmp = $result['blog'];
            unset($result['blog']);
            $result['blog'][] = $tmp;
        }
        return $result['blog'];
    }

    function blogs_postPhoto($blog_id, $photo_id, $title, $description, $blog_password = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.blogs.postPhoto.html */
        $this->request('flickr.blogs.postPhoto', array('blog_id'=>$blog_id, 'photo_id'=>$photo_id, 'title'=>$title, 'description'=>$description, 'blog_password'=>$blog_password), TRUE);
        $this->parse_response();
        return true;
    }

    /* Contacts Methods */
    function contacts_getList ($filter = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.contacts.getList.html */
        $this->request('flickr.contacts.getList', array('filter'=>$filter));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['contacts'];
        if (!empty($result['contact']['nsid'])) {
            $tmp = $result['contact'];
            unset($result['contact']);
            $result['contact'][] = $tmp;
        }
        return $result['contact'];
    }

    function contacts_getPublicList($user_id)
    {
        /* http://www.flickr.com/services/api/flickr.contacts.getPublicList.html */
        $this->request('flickr.contacts.getPublicList', array('user_id'=>$user_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['contacts'];
        if (!empty($result['contact']['nsid'])) {
            $tmp = $result['contact'];
            unset($result['contact']);
            $result['contact'][] = $tmp;
        }
        return $result['contact'];
    }

    /* Favorites Methods */
    function favorites_add ($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.favorites.add.html */
        $this->request('flickr.favorites.add', array('photo_id'=>$photo_id), TRUE);
        $this->parse_response();
        return true;
    }

    function favorites_getList($user_id = NULL, $extras = NULL, $per_page = NULL, $page = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.favorites.getList.html */
        if (is_array($extras)) { $extras = implode(",", $extras); }
        $this->request("flickr.favorites.getList", array("user_id"=>$user_id, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function favorites_getPublicList($user_id = NULL, $extras = NULL, $per_page = NULL, $page = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.favorites.getPublicList.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }
        $this->request("flickr.favorites.getPublicList", array("user_id"=>$user_id, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function favorites_remove($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.favorites.remove.html */
        $this->request("flickr.favorites.remove", array("photo_id"=>$photo_id), TRUE);
        $this->parse_response();
        return true;
    }

    /* Groups Methods */
    function groups_browse ($cat_id = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.groups.browse.html */
        $this->request("flickr.groups.browse", array("cat_id"=>$cat_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['category'];
        if (!empty($result['subcat']['id'])) {
            $tmp = $result['subcat'];
            unset($result['subcat']);
            $result['subcat'][] = $tmp;
        }
        if (!empty($result['group']['nsid'])) {
            $tmp = $result['group'];
            unset($result['group']);
            $result['group'][] = $tmp;
        }
        return $result;
    }

    function groups_getInfo ($group_id)
    {
        /* http://www.flickr.com/services/api/flickr.groups.getInfo.html */
        $this->request("flickr.groups.getInfo", array("group_id"=>$group_id));
        $this->parse_response();
        return $this->parsed_response['rsp']["group"];
    }

    /* Groups Methods */
    function groups_pools_add ($photo_id, $group_id)
    {
        /* http://www.flickr.com/services/api/flickr.groups.pools.add.html */
        $this->request("flickr.groups.pools.add", array("photo_id"=>$photo_id, "group_id"=>$group_id), TRUE);
        $this->parse_response();
        return true;
    }

    function groups_pools_getContext ($photo_id, $group_id)
    {
        /* http://www.flickr.com/services/api/flickr.groups.pools.getContext.html */
        $this->request("flickr.groups.pools.getContext", array("photo_id"=>$photo_id, "group_id"=>$group_id));
        $this->parse_response();
        return $this->parsed_response['rsp'];
    }

    function groups_pools_getGroups ()
    {
        /* http://www.flickr.com/services/api/flickr.groups.pools.getGroups.html */
        $this->request("flickr.groups.pools.getGroups");
        $this->parse_response();
        $result = $this->parsed_response['rsp']['groups']['group'];
        if (!empty($result['id'])) {
            $tmp = $result;
            unset($result);
            $result[] = $tmp;
        }
        return $result;
    }

    function groups_pools_getPhotos ($group_id, $tags = NULL, $user_id = NULL, $extras = NULL, $per_page = NULL, $page = NULL)
    {
		/* http://www.flickr.com/services/api/flickr.groups.pools.getPhotos.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }
        $this->request("flickr.groups.pools.getPhotos", array("group_id"=>$group_id, "tags"=>$tags, "user_id"=>$user_id, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function groups_pools_remove ($photo_id, $group_id)
    {
        /* http://www.flickr.com/services/api/flickr.groups.pools.remove.html */
        $this->request("flickr.groups.pools.remove", array("photo_id"=>$photo_id, "group_id"=>$group_id), TRUE);
        $this->parse_response();
        return true;
    }

	function groups_search ($text, $per_page=NULL, $page=NULL)
	{
		/* http://www.flickr.com/services/api/flickr.groups.search.html */
		$this->request("flickr.groups.search", array("text"=>$text,"per_page"=>$per_page,"page"=>$page));
		$this->parse_response();
		$result = $this->parsed_response['rsp']['groups'];
		if (!empty($result['group']['nsid'])) {
			$tmp = $result['group'];
			unset($result['group']);
			$result['group'][] = $tmp;
		}
		return $result;
	}

    /* Interestingness methods */
	function interestingness_getList($date = NULL, $extras = NULL, $per_page = NULL, $page = NULL)
	{
        /* http://www.flickr.com/services/api/flickr.interestingness.getList.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }

        $this->request("flickr.interestingness.getList", array("date"=>$date, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    /* People methods */
    function people_findByEmail ($find_email)
    {
        /* http://www.flickr.com/services/api/flickr.people.findByEmail.html */
        $this->request("flickr.people.findByEmail", array("find_email"=>$find_email));
        $this->parse_response();
        return $this->parsed_response['rsp']["user"]["nsid"];
    }

    function people_findByUsername ($username)
    {
        /* http://www.flickr.com/services/api/flickr.people.findByUsername.html */
        $this->request("flickr.people.findByUsername", array("username"=>$username));
        $this->parse_response();
        return $this->parsed_response['rsp']["user"]["nsid"];
    }

    function people_getInfo($user_id)
    {
        /* http://www.flickr.com/services/api/flickr.people.getInfo.html */
        $this->request("flickr.people.getInfo", array("user_id"=>$user_id));
        $this->parse_response();
        return $this->parsed_response['rsp']["person"];
    }

    function people_getPublicGroups($user_id)
    {
        /* http://www.flickr.com/services/api/flickr.people.getPublicGroups.html */
        $this->request("flickr.people.getPublicGroups", array("user_id"=>$user_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['groups'];
        if (!empty($result['group']['nsid'])) {
            $tmp = $result['group'];
            unset($result['group']);
            $result['group'][] = $tmp;
        }
        return $result;
    }

    function people_getPublicPhotos($user_id, $extras = NULL, $per_page = NULL, $page = NULL) {
        /* http://www.flickr.com/services/api/flickr.people.getPublicPhotos.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }

        $this->request("flickr.people.getPublicPhotos", array("user_id"=>$user_id, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function people_getUploadStatus($user_id)
    {
        /* http://www.flickr.com/services/api/flickr.people.getUploadStatus.html */
        /* Requires Authentication */
        $this->request("flickr.people.getUploadStatus");
        $this->parse_response();
        return $this->parsed_response['rsp']['user'];
    }


    /* Photos Methods */
    function photos_addTags ($photo_id, $tags)
    {
        /* http://www.flickr.com/services/api/flickr.photos.addTags.html */
        $this->request("flickr.photos.addTags", array("photo_id"=>$photo_id, "tags"=>$tags), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_delete($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.delete.html */
        $this->request("flickr.photos.delete", array("photo_id"=>$photo_id), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_getAllContexts ($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getAllContexts.html */
        $this->request("flickr.photos.getAllContexts", array("photo_id"=>$photo_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp'];
        if (!empty($result['set']['id'])) {
            $tmp = $result['set'];
            unset($result['set']);
            $result['set'][] = $tmp;
        }
        if (!empty($result['pool']['id'])) {
            $tmp = $result['pool'];
            unset($result['pool']);
            $result['pool'][] = $tmp;
        }
        return $result;    }

    function photos_getContactsPhotos ($count = NULL, $just_friends = NULL, $single_photo = NULL, $include_self = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getContactsPhotos.html */
        $this->request("flickr.photos.getContactsPhotos", array("count"=>$count, "just_friends"=>$just_friends, "single_photo"=>$single_photo, "include_self"=>$include_self));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result['photo'];
    }

    function photos_getContactsPublicPhotos ($user_id, $count = NULL, $just_friends = NULL, $single_photo = NULL, $include_self = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getContactsPublicPhotos.html */
        $this->request("flickr.photos.getContactsPublicPhotos", array("user_id"=>$user_id, "count"=>$count, "just_friends"=>$just_friends, "single_photo"=>$single_photo, "include_self"=>$include_self));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result['photo'];
    }

    function photos_getContext ($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getContext.html */
        $this->request("flickr.photos.getContext", array("photo_id"=>$photo_id));
        $this->parse_response();
        return $this->parsed_response['rsp'];
    }

    function photos_getCounts ($dates = NULL, $taken_dates = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getCounts.html */
        $this->request("flickr.photos.getCounts", array("dates"=>$dates, "taken_dates"=>$taken_dates));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photocounts'];
        if (!empty($result['photocount']['count'])) {
            $tmp = $result['photocount'];
            unset($result['photocount']);
            $result['photocount'][] = $tmp;
        }
        return $result['photocount'];
    }

    function photos_getExif ($photo_id, $secret = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getExif.html */
        $this->request("flickr.photos.getExif", array("photo_id"=>$photo_id, "secret"=>$secret));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photo'];
        if (!empty($result['exif']['tagspace'])) {
            $tmp = $result['exif'];
            unset($result['exif']);
            $result['exif'][] = $tmp;
        }
        return $result;
    }

    function photos_getInfo($photo_id, $secret = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getInfo.html */
        $this->request("flickr.photos.getInfo", array("photo_id"=>$photo_id, "secret"=>$secret));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photo'];
        if (!empty($result['urls']['url']['type'])) {
            $tmp = $result['urls']['url'];
            unset($result['urls']['url']);
            $result['urls']['url'][] = $tmp;
        }
        if (!empty($result['tags']['tag']['id'])) {
            $tmp = $result['tags']['tag'];
            unset($result['tags']['tag']);
            $result['tags']['tag'][] = $tmp;
        }
        if (!empty($result['notes']['note']['id'])) {
            $tmp = $result['notes']['note'];
            unset($result['notes']['note']);
            $result['notes']['note'][] = $tmp;
        }
        return $result;
    }

    function photos_getNotInSet($min_upload_date = NULL, $max_upload_date = NULL, $min_taken_date = NULL, $max_taken_date = NULL, $extras = NULL, $per_page = NULL, $page = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getNotInSet.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }
        $this->request("flickr.photos.getNotInSet", array("min_upload_date"=>$min_upload_date, "max_upload_date"=>$max_upload_date, "min_taken_date"=>$min_taken_date, "max_taken_date"=>$max_taken_date, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']["photos"];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function photos_getPerms($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getPerms.html */
        $this->request("flickr.photos.getPerms", array("photo_id"=>$photo_id));
        $this->parse_response();
        return $this->parsed_response['rsp']["perms"];
    }

    function photos_getRecent($extras = NULL, $per_page = NULL, $page = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getRecent.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }

        $this->request("flickr.photos.getRecent", array("extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function photos_getSizes($photo_id, $secret = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getSizes.html */
        $this->request("flickr.photos.getSizes", array("photo_id"=>$photo_id));
        $this->parse_response();
        foreach($this->parsed_response['rsp']["sizes"]['size'] as $size) {
            $result[$size["label"]] = $size;
        }
        return $result;
    }

    function photos_getUntagged($min_upload_date = NULL, $max_upload_date = NULL, $min_taken_date = NULL, $max_taken_date = NULL, $extras = NULL, $per_page = NULL, $page = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.getUntagged.html */
        if (is_array($extras)) {
            $extras = implode(",", $extras);
        }
        $this->request("flickr.photos.getUntagged", array("min_upload_date"=>$min_upload_date, "max_upload_date"=>$max_upload_date, "min_taken_date"=>$min_taken_date, "max_taken_date"=>$max_taken_date, "extras"=>$extras, "per_page"=>$per_page, "page"=>$page));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function photos_removeTag($tag_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.removeTag.html */
        $this->request("flickr.photos.removeTag", array("tag_id"=>$tag_id), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_search($args)
    {
        /* This function strays from the method of arguments that I've
         * used in the other functions for the fact that there are just
         * so many arguments to this API method. What you'll need to do
         * is pass an associative array to the function containing the
         * arguments you want to pass to the API.  For example:
         *   $photos = $f->photos_search(array("tags"=>"brown,cow", "tag_mode"=>"any"));
         * This will return photos tagged with either "brown" or "cow"
         * or both. See the API documentation (link below) for a full
         * list of arguments.
         */

        /* http://www.flickr.com/services/api/flickr.photos.search.html */
        $this->request("flickr.photos.search", $args);
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photos'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function photos_setDates($photo_id, $date_posted = NULL, $date_taken = NULL, $date_taken_granularity = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photos.setDates.html */
        $this->request("flickr.photos.setDates", array("photo_id"=>$photo_id, "date_posted"=>$date_posted, "date_taken"=>$date_taken, "date_taken_granularity"=>$date_taken_granularity), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_setMeta($photo_id, $title, $description)
    {
        /* http://www.flickr.com/services/api/flickr.photos.setMeta.html */
        $this->request("flickr.photos.setMeta", array("photo_id"=>$photo_id, "title"=>$title, "description"=>$description), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_setPerms($photo_id, $is_public, $is_friend, $is_family, $perm_comment, $perm_addmeta)
    {
        /* http://www.flickr.com/services/api/flickr.photos.setPerms.html */
        $this->request("flickr.photos.setPerms", array("photo_id"=>$photo_id, "is_public"=>$is_public, "is_friend"=>$is_friend, "is_family"=>$is_family, "perm_comment"=>$perm_comment, "perm_addmeta"=>$perm_addmeta), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_setTags($photo_id, $tags)
    {
        /* http://www.flickr.com/services/api/flickr.photos.setTags.html */
        $this->request("flickr.photos.setTags", array("photo_id"=>$photo_id, "tags"=>$tags), TRUE);
        $this->parse_response();
        return true;
    }

    /* Photos - Comments Methods */
    function photos_comments_getList($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.comments.getList.html */
        $this->request("flickr.photos.comments.getList", array("photo_id"=>$photo_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['comments'];
        if (!empty($result['comment']['id'])) {
            $tmp = $result['comment'];
            unset($result['comment']);
            $result['comment'][] = $tmp;
        }
        return $result['comment'];
    }

    /* Photos - Notes Methods */
    function photos_licenses_getInfo()
    {
        /* http://www.flickr.com/services/api/flickr.photos.licenses.getInfo.html */
        $this->request("flickr.photos.licenses.getInfo");
        $this->parse_response();
        return $this->parsed_response['rsp']['licenses'];
    }

    function photos_licenses_setLicense($photo_id, $license_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.licenses.setLicense.html */
        /* Requires Authentication */
        $this->request("flickr.photos.licenses.setLicense", array("photo_id"=>$photo_id, "license_id"=>$license_id), TRUE);
        $this->parse_response();
        return true;
    }

    /* Photos - Notes Methods */
    function photos_notes_add($photo_id, $note_x, $note_y, $note_w, $note_h, $note_text)
    {
        /* http://www.flickr.com/services/api/flickr.photos.notes.add.html */
        $this->request("flickr.photos.notes.add", array("photo_id" => $photo_id, "note_x" => $note_x, "note_y" => $note_y, "note_w" => $note_w, "note_h" => $note_h, "note_text" => $note_text), TRUE);
        $this->parse_response();
        return $this->parsed_response['rsp']['note']['id'];
    }

    function photos_notes_delete($note_id)
    {
        /* http://www.flickr.com/services/api/flickr.photos.notes.delete.html */
        $this->request("flickr.photos.notes.delete", array("note_id" => $note_id), TRUE);
        $this->parse_response();
        return true;
    }

    function photos_notes_edit($note_id, $note_x, $note_y, $note_w, $note_h, $note_text)
    {
        /* http://www.flickr.com/services/api/flickr.photos.notes.edit.html */
        $this->request("flickr.photos.notes.edit", array("note_id" => $note_id, "note_x" => $note_x, "note_y" => $note_y, "note_w" => $note_w, "note_h" => $note_h, "note_text" => $note_text), TRUE);
        $this->parse_response();
        return true;
    }

    /* Photos - Transform Methods */
    function photos_transform_rotate($photo_id, $degrees)
    {
        /* http://www.flickr.com/services/api/flickr.photos.transform.rotate.html */
        $this->request("flickr.photos.transform.rotate", array("photo_id" => $photo_id, "degrees" => $degrees), TRUE);
        $this->parse_response();
        return true;
    }

    /* Photos - Upload Methods */
    function photos_upload_checkTickets($tickets)
    {
        /* http://www.flickr.com/services/api/flickr.photos.upload.checkTickets.html */
        if (is_array($tickets)) {
            $tickets = implode(",", $tickets);
        }
        $this->request("flickr.photos.upload.checkTickets", array("tickets" => $tickets), TRUE);
        $result = $this->parse_response();
        if (!empty($result['uploader']['ticket']['id'])) {
            $tmp = $result['uploader']['ticket'];
            unset($result['uploader']['ticket']);
            $result['uploader']['ticket'][] = $tmp;
        }
        return $result['uploader']['ticket'];
    }

    /* Photosets Methods */
    function photosets_addPhoto($photoset_id, $photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.addPhoto.html */
        $this->request("flickr.photosets.addPhoto", array("photoset_id" => $photoset_id, "photo_id" => $photo_id), TRUE);
        $this->parse_response();
        return true;
    }

    function photosets_create($title, $description, $primary_photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.create.html */
        $this->request("flickr.photosets.create", array("title" => $title, "primary_photo_id" => $primary_photo_id, "description" => $description), TRUE);
        $this->parse_response();
        return $this->parsed_response['rsp']['photoset'];
    }

    function photosets_delete($photoset_id)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.delete.html */
        $this->request("flickr.photosets.delete", array("photoset_id" => $photoset_id), TRUE);
        $this->parse_response();
        return true;
    }

    function photosets_editMeta($photoset_id, $title, $description = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.editMeta.html */
        $this->request("flickr.photosets.editMeta", array("photoset_id" => $photoset_id, "title" => $title, "description" => $description), TRUE);
        $this->parse_response();
        return true;
    }

    function photosets_editPhotos($photoset_id, $primary_photo_id, $photo_ids)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.editPhotos.html */
        $this->request("flickr.photosets.editPhotos", array("photoset_id" => $photoset_id, "primary_photo_id" => $primary_photo_id, "photo_ids" => $photo_ids), TRUE);
        $this->parse_response();
        return true;
    }

    function photosets_getContext($photo_id, $photoset_id)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.getContext.html */
        $this->request("flickr.photosets.getContext", array("photo_id" => $photo_id, "photoset_id" => $photoset_id));
        $this->parse_response();
        return $this->parsed_response['rsp'];
    }

    function photosets_getInfo($photoset_id)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.getInfo.html */
        $this->request("flickr.photosets.getInfo", array("photoset_id" => $photoset_id));
        $this->parse_response();
        return $this->parsed_response['rsp']['photoset'];
    }

    function photosets_getList($user_id = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.getList.html */
        $this->request("flickr.photosets.getList", array("user_id" => $user_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photosets'];
        if (!empty($result['photoset']['id'])) {
            $tmp = $result['photoset'];
            unset($result['photoset']);
            $result['photoset'][] = $tmp;
        }
        return $result;
    }

    function photosets_getPhotos($photoset_id, $extras = NULL, $privacy_filter = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.getPhotos.html */
        $this->request("flickr.photosets.getPhotos", array("photoset_id" => $photoset_id, "extras" => $extras, "privacy_filter" => $privacy_filter));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photoset'];
        if (!empty($result['photo']['id'])) {
            $tmp = $result['photo'];
            unset($result['photo']);
            $result['photo'][] = $tmp;
        }
        return $result;
    }

    function photosets_orderSets($photoset_ids)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.orderSets.html */
        if (is_array($photoset_ids)) {
            $photoset_ids = implode(",", $photoset_ids);
        }
        $this->request("flickr.photosets.orderSets", array("photoset_ids" => $photoset_ids), TRUE);
        $this->parse_response();
        return true;
    }

    function photosets_removePhoto($photoset_id, $photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.photosets.removePhoto.html */
        $this->request("flickr.photosets.removePhoto", array("photoset_id" => $photoset_id, "photo_id" => $photo_id), TRUE);
        $this->parse_response();
        return true;
    }

    /* Reflection Methods */
    function reflection_getMethodInfo($method_name)
    {
        /* http://www.flickr.com/services/api/flickr.reflection.getMethodInfo.html */
        $this->request("flickr.reflection.getMethodInfo", array("method_name" => $method_name));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['method'];
        if (!empty($result['arguments']['argument']['name'])) {
            $tmp = $result['arguments']['argument'];
            unset($result['arguments']['argument']);
            $result['arguments']['argument'][] = $tmp;
        }
        if (!empty($result['errors']['error']['code'])) {
            $tmp = $result['errors']['error'];
            unset($result['errors']['error']);
            $result['errors']['error'][] = $tmp;
        }
        return $result;
    }

    function reflection_getMethods()
    {
        /* http://www.flickr.com/services/api/flickr.reflection.getMethods.html */
        $this->request("flickr.reflection.getMethods");
        $this->parse_response();
        return $this->parsed_response['rsp']['methods'];
    }

    /* Tags Methods */
    function tags_getListPhoto($photo_id)
    {
        /* http://www.flickr.com/services/api/flickr.tags.getListPhoto.html */
        $this->request("flickr.tags.getListPhoto", array("photo_id" => $photo_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['photo'];
        if (!empty($result['tags']['tag']['id'])) {
            $tmp = $result['tags']['tag'];
            unset($result['tags']['tag']);
            $result['tags']['tag'][] = $tmp;
        }
        return $result;
    }

    function tags_getListUser($user_id = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.tags.getListUser.html */
        $this->request("flickr.tags.getListUser", array("user_id" => $user_id));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['who'];
        if (!is_array($result['tags']['tag'])) {
            $tmp = $result['tags']['tag'];
            unset($result['tags']['tag']);
            $result['tags']['tag'][] = $tmp;
        }
        return $result;
    }

    function tags_getListUserPopular($user_id = NULL, $count = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.tags.getListUserPopular.html */
        $this->request("flickr.tags.getListUserPopular", array("user_id" => $user_id, "count" => $count));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['who'];
        if (!empty($result['tags']['tag']['count'])) {
            $tmp = $result['tags']['tag'];
            unset($result['tags']['tag']);
            $result['tags']['tag'][] = $tmp;
        }
        return $result;
    }

    function tags_getRelated($tag)
    {
        /* http://www.flickr.com/services/api/flickr.tags.getRelated.html */
        $this->request("flickr.tags.getRelated", array("tag" => $tag));
        $this->parse_response();
        $result = $this->parsed_response['rsp']['tags'];
        if (!is_array($result['tag'])) {
            $tmp = $result['tag'];
            unset($result['tag']);
            $result['tag'][] = $tmp;
        }
        return $result;
    }

    function test_echo($args = array())
    {
        /* http://www.flickr.com/services/api/flickr.test.echo.html */
        $this->request("flickr.test.echo", $args);
        $this->parse_response();
        return $this->parsed_response['rsp'];
    }

    function test_login()
    {
        /* http://www.flickr.com/services/api/flickr.test.login.html */
        $this->request("flickr.test.login");
        $this->parse_response();
        return $this->parsed_response['rsp']['user'];
    }

    function urls_getGroup($group_id)
    {
        /* http://www.flickr.com/services/api/flickr.urls.getGroup.html */
        $this->request("flickr.urls.getGroup", array("group_id"=>$group_id));
        $this->parse_response();
        return $this->parsed_response['rsp']["group"]["url"];
    }

    function urls_getUserPhotos($user_id = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.urls.getUserPhotos.html */
        $this->request("flickr.urls.getUserPhotos", array("user_id"=>$user_id));
        $this->parse_response();
        return $this->parsed_response['rsp']["user"]["url"];
    }

    function urls_getUserProfile($user_id = NULL)
    {
        /* http://www.flickr.com/services/api/flickr.urls.getUserProfile.html */
        $this->request("flickr.urls.getUserProfile", array("user_id"=>$user_id));
        $this->parse_response();
        return $this->parsed_response['rsp']["user"]["url"];
    }

    function urls_lookupGroup($url)
    {
        /* http://www.flickr.com/services/api/flickr.urls.lookupGroup.html */
        $this->request("flickr.urls.lookupGroup", array("url"=>$url));
        $this->parse_response();
        return $this->parsed_response['rsp']["group"];
    }

    function urls_lookupUser($url)
    {
        /* http://www.flickr.com/services/api/flickr.photos.notes.edit.html */
        $this->request("flickr.urls.lookupUser", array("url"=>$url));
        $this->parse_response();
        return $this->parsed_response['rsp']["user"];
    }
}


?>
