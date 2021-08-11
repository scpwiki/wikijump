<?php

namespace Ozone\Framework;




use Ozone\Framework\Database\Criteria;
use Ozone\Framework\DB\OzoneSession;
use Wikidot\DB\OzoneSessionPeer;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Models\User;

/**
 * Class containing most important properties of the request/response.
 */
class RunData {

	private $parameterList;
	private $screenTemplate;
	private $screenClassName;
	private $screenClassPath;

	// used only when processing a module
	private $moduleTemplate;
	private $moduleClassName;
	private $moduleClassPath;

	private $context;
	private $action;
	private $actionEvent;
	private $nextAction;
	private $nextActionEvent;
	private $errorMessages = array ();
	private $messages = array ();
	private $page;
	private $cookies;
	private $language;

	private $session = null;

	private $ajaxMode = false;
	private $ajaxResponse;

	private $requestUri;
	private $requestMethod;

	private $extra = array();

	private $formToolHttpProcessed = false;

	private $temp; // temporary variables

	private $_outCookies = array();

	/**
	 * Default constructor.
	 */
	public function __construct() {
		$this->page = new PageProperties();
	}

    /**
     * Helper/shortcut methods for RunData:
     */

    /**
     * Get the user ID of the calling user.
     * @return int|null
     */
    public function id() : ?int
    {
        return $this->getUserId();
    }

    /**
     * Shortcut method to retrieve a value from a RunData object.
     * @param string $key
     * @return mixed
     */
    public function get(string $key)
    {
        return $this->getParameterList()->getParameterValue($key);
    }

    /**
     * Retrieve the calling User model.
     * @return User|null
     */
    public function user(): ?User
    {
        return $this->getUser();
    }

	/**
		* Initializes a RunData object.
		*/
	public function init() {

		$parameterList = new ParameterList();
		$parameterList->initParameterList($this);
		$this->parameterList = $parameterList;

		$this->setTemplateFromParameterList();
		//set action
		$action =  $this->parameterList->getParameterValue('action');

		$parameterArray = $this->parameterList->asArray();
		// now parse some importand parameters: language, skin

		if ($parameterArray["lang"] != null) {
			$this->language = $parameterArray["lang"];
		} else {
			$this->language = GlobalProperties :: $DEFAULT_LANGUAGE;
		}

		if ($parameterArray["skin"] != null) {
			$this->page->setSkin($parameterArray["skin"]);
		}

		if ($action !== null  && preg_match('/^[a-z0-9_\/]+$/i', $action) == 1) {
			$this->parameterList->delParameter['action'];
			$this->action = str_replace("__", "/",$action);

			// set action event
			// this on is more complicated - extract event from a key in the parameter list
			// of the form event_someevent

			//first check if event=foobar is present
			foreach ($parameterArray as $key => $value) {
				if ($key == 'event') {
					$this->actionEvent = $value.'Event';
				}
			}

			foreach ($parameterArray as $key => $value) {
				if (preg_match('/event_/', $key)) {
					$this->actionEvent = str_replace('event_', '', $key).'Event';
					break;
				}
			}
		}

		if (! preg_match(';\.' . GlobalProperties::$URL_DOMAIN_PREG . '$;', $_SERVER['HTTP_HOST'])) {
			GlobalProperties::$SESSION_COOKIE_NAME .= "_" . substr(md5($_SERVER['HTTP_HOST']), 3, 8);
			GlobalProperties::$SESSION_COOKIE_DOMAIN = '.' . $_SERVER['HTTP_HOST'];
		}

		// initialize cookies...
		$this->cookies = $_COOKIE;

		// store original request uri and request method:
		$this->requestUri = $_SERVER['REQUEST_URI'];
		$this->requestMethod = $_SERVER['REQUEST_METHOD'];

	}

	public function getRequestUri() {
		return $this->requestUri;
	}

	public function getRequestMethod() {
		return $this->requestMethod;

	}

	public function getScreenClassName() {
		return $this->screenClassName;
	}

	public function getScreenClassPath() {
		return $this->screenClassPath;
	}

	public function getParameterList() {
		return $this->parameterList;
	}

	public function setParameterList($parameterList) {
		$this->parameterList = $parameterList;
	}

	public function getPage() {
		return $this->page;
	}

	public function setPage($page) {
		$this->page = $page;
	}

	public function getSession() {
		return $this->session;
	}

	public function setSession($var) {
		$this->session = $var;
	}

	public function setAction($action) {
		$this->action = $action;
	}

	public function getAction() {
		return $this->action;
	}

	public function getActionEvent() {
		return $this->actionEvent;
	}

	public function setActionEvent($actionEvent) {
		$this->actionEvent = $actionEvent;
	}

	public function setNextAction($action) {
		$this->nextAction = $action;
	}

	public function getNextAction() {
		return $this->nextAction;
	}

	public function getNextActionEvent() {
		return $this->nextActionEvent;
	}

	public function setNextActionEvent($actionEvent) {
		$this->nextActionEvent = $actionEvent;
	}

	public function setScreenTemplate2($screenTemplate) {
		$this->screenTemplate = str_replace(',', '/', $screenTemplate);
	}

	public function setTemplateScreenFromGetPost($getArray, $postArray) {

		if (array_key_exists('template', $postArray)) {
			$this->screenTemplate = str_replace(',', '/', $postArray['template']);
		} else
			if (array_key_exists('template', $getArray)) {
				$this->screenTemplate = str_replace(',', '/', $getArray['template']);
			} else {
				$this->screenTemplate = "Index";

			}
		$this->findClass();
	}

	private function setTemplateFromParameterList() {
		if(!$this->ajaxMode){
			// normal mode
			$template = $this->parameterList->getParameterValue("template");
			if ($template == null || preg_match('/^[a-z0-9_\/]+$/i',$template) != 1) {
				$template = "Index";
			}
			$template = str_replace("__", "/", $template);
			$this->screenTemplate = $template;
			$this->findClass();
		} else {
			// ajax call mode
			$template = $this->parameterList->getParameterValue("moduleName");
			if ($template == null || preg_match('/^[a-z0-9_\/]+$/i',$template) != 1) {
				$template = "Empty";
			}
			$this->moduleTemplate = $template;
			$this->findClass();
		}
	}

	public function getScreenTemplateRaw() {
		return $this->screenTemplate;
	}

	public function getScreenTemplate() {
		return str_replace(',', '/', $this->screenTemplate);

	}

	public function getModuleTemplate() {
		return  $this->moduleTemplate;
	}

	public function setModuleTemplate($template) {
		$this->moduleTemplate = $template;
		$this->findClass();
	}

	public function getModuleClassPath() {
		return  $this->moduleClassPath;
	}

	public function getModuleClassName() {
		return  $this->moduleClassName;
	}

	public function setScreenTemplate($template) {
		$this->screenTemplate = $template;
		$this->findClass();
	}

	public function addErrorMessage($message) {
		$this->errorMessages[] = $message;
	}

	public function addMessage($message) {
		$this->messages[] = $message;
	}

	public function getErrorMessages() {
		return $this->errorMessages;
	}

	public function getMessages() {
		return $this->messages;
	}

	public function contextDel($key = null) {
		if ($key != null) {
			unset ($this->context["$key"]);
		} else {
			$this->context = array ();
		}
	}

	public function contextAdd($key, $value) {
		$this->context["$key"] = $value;
	}

	public function contextGet($key) {
		return $this->context["$key"];
	}

	public function getContext() {
		return $this->context;
	}

	public function setContext($context){
		$this->context = $context;
	}

	public function getLanguage() {
		return $this->language;
	}

	public function setLanguage($lang){
		$this->language = $lang;
	}

	public function setAjaxMode($val){
		$this->ajaxMode = $val;
	}

	public function getAjaxMode(){
		return $this->ajaxMode;
	}
	public function isAjaxMode(){
		return $this->ajaxMode;
	}

	/**
	 * Finds Class given the template name.
	 */
	private function findClass() {
		if(!$this->ajaxMode){
		$classFilename = PathManager :: screenClass($this->screenTemplate);

		if (file_exists($classFilename)) {
			$this->screenClassPath = $classFilename;
			$tmp1 = explode('/', $this->screenTemplate);
			$size = sizeof($tmp1);
			$this->screenClassName = $tmp1[$size -1];

		} else {
			$tmppath = PathManager :: screenClassDir();
			## generate list of possible classes:
			$template = $this->screenTemplate;
			$path44 = explode('/', $template);

			for ($i = sizeof($path44) - 1; $i >= 0; $i --) {

				$tmppath2 = "";
				for ($k = 0; $k < $i; $k ++) {
					$tmppath2 .= $path44[$k]."/";
				}
				$tmppath2 .= "DefaultScreen.php";
				$classFiles[] = $tmppath2;
			}

			foreach ($classFiles as $classFile) {
				if (file_exists($tmppath.$classFile)) {
					$this->screenClassPath = $tmppath.$classFile;
					$this->screenClassName = "DefaultScreen";
					break;
				}
			}

		}
		} else {
			$ttt = ModuleHelpers::findModuleClass($this->moduleTemplate);
			$this->moduleClassName = $ttt[0];
			$this->moduleClassPath = $ttt[1];
		}

	}

	/**
	 * Start handling session. If session does not exist - start one. If exists - do nothing.
	 */
	public function sessionStart(){
		if($this->session == null){
			// create a new session

			$sessionId = UniqueStrings::random_string(60);
			$cookieKey = GlobalProperties::$SESSION_COOKIE_NAME;
			$this->_setCookie($cookieKey, $sessionId, time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
			$session = new OzoneSession();

			// set IP
			$session->setIpAddress($this->createIpString());

			// set UA hash
			$session->setUaHash($this->createUaHash());

			// set unique SESSION_ID
			$session->setSessionId($sessionId);

			$date = new ODate();
			$session->setStarted($date);
			$session->setLastAccessed($date);

			$session->setNewSession(true);
			$session->setUserId(null); // will this work?
			$this->session = $session;
		}
	}

	/**
	 * Stops handling session - removing the cookie etc.
	 *
	 */
	public function sessionStop($removeCookie = true){
		$s = $this->getSession();
		if ($s) {
			$memcache = Ozone::$memcache;
			$mkey = 'session..'.$s->getSessionId();
			$memcache->delete($mkey);

			OzoneSessionPeer :: instance()->deleteByPrimaryKey($s->getSessionId());
			$this->session = null;

		}
		if($removeCookie){
			$cookieKey = GlobalProperties::$SESSION_COOKIE_NAME;
			$this->_setCookie($cookieKey, 'dummy', time() - 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
		}
	}

	public function getSessionFromDomainHash($session_hash, $domain, $user_id) {
		$domain = pg_escape_string(strtolower($domain));
		$user_id = (int) $user_id;
		$session_hash = pg_escape_string($session_hash);
		$secret = pg_escape_string(GlobalProperties::$SECRET_DOMAIN_LOGIN);

		$c = new Criteria();
		$c->add("user_id", $user_id);
		$c->add("MD5('${domain}_${secret}_' || session_id)", $session_hash);
		$session = OzoneSessionPeer::instance()->selectOne($c);

		return $session;
	}

	public function generateSessionDomainHash($domain) {
		$domain = strtolower($domain);
		$user_id = $this->getUserId();
		$session_id = $this->getSessionId();
		$secret = GlobalProperties::$SECRET_DOMAIN_LOGIN;

		return md5("${domain}_${secret}_${session_id}");
	}

	/**
	 * Handle session at the beginning of the request procession.
	 */
	public function handleSessionStart() {
		// check if session cookie exists
		$cookieKey = GlobalProperties::$SESSION_COOKIE_NAME;
		$cookieSessionId = $this->cookies[$cookieKey];

		// TODO: we can optimise this a bit... like don't fetch the session the second time from db
		$m = array();
		if (preg_match(";^_domain_cookie_(.*)_(.*)$;", $cookieSessionId, $m)) {
			$user_id = (int) $m[1];
			$session_hash = $m[2];
			$domain = $_SERVER['HTTP_HOST'];

			$session_from_db = $this->getSessionFromDomainHash($session_hash, $domain, $user_id);

			if ($session_from_db) {
				$cookieSessionId = $session_from_db->getSessionId();
			}
		}

		if ($cookieSessionId == false || $cookieSessionId == '' || !$cookieSessionId) {
			// no session cookie, we do not force one (new cool policy).
			return ;
		}
		//ok, cookie is here. check if corresponds to a valid session
		// try memcached first
		$memcache = Ozone::$memcache;
		$mkey = 'session..'.$cookieSessionId;

		$session = $memcache->get($mkey);
		if(!$session){
			$session = OzoneSessionPeer :: instance()->selectByPrimaryKey($cookieSessionId);
		}
		if(!$session){
			// no session object, delete the cookie!
			$this->_setCookie($cookieKey, $cookieSessionId, time() - 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
			return;
		}

		// if we are here it means that the session object EXISTS in the database. now see if it is
		// valid. if ok - leave it. if not - clean up.
		$sessionValid = true;

		if ($session->getInfinite() == false) {

			$minTimestamp = new ODate();
			$minTimestamp->subtractSeconds(GlobalProperties :: $SESSION_TIMEOUT);

			if ($session->getLastAccessed()->before($minTimestamp)) {
				$sessionValid = false;
			}

		}

		if ($session->getCheckIp() == true) {
			$currentIpString = $this->createIpString();
			if($_SERVER['HTTPS'] && $session->getIpAddressSsl()){
				$sessionIpString = $session->getIpAddressSsl();
			} else {
				$sessionIpString = $session->getIpAddress();
			}
			if ($currentIpString != $sessionIpString) {
				$sessionValid = false;
				$this->session = null;
				return; // nasty, we should not remove this session.
			}
		}

		/* Check UA hash. */

		if($session->getUaHash() != $this->createUaHash()){
			$sessionValid = false;
			$this->session = null;
			return;
		}

		if($sessionValid == false){
			// cleanup again
			$c = new Criteria();
			$c->add("session_id", $session->getSessionId());
			OzoneSessionPeer :: instance()->delete($c);
			$memcache->delete($mkey);
		}else {

			// 	all is right, set the session now.
			$this->session = $session;
		}
		return;

	}

	/**
	 * Handle session at the end of the request procession.
	 */
	public function handleSessionEnd() {
	    if(!$this->session) { return; }
			// if session storage is empty and userId = null - clear stop the session!
			$session = $this->session;
			$serializedData = $session->getSerializedData();
			if($serializedData === false) { $serializedData = []; }
			if(!$this->getUser() && count($serializedData) == 0){
				$this->sessionStop();
			} else{
				$date = new ODate();
				$session->setLastAccessed($date);

				// save it to the database too?
				$lastSavedDate = $session->getTemp("lastSaved");
				if($session->getSessionChanged()
						|| !$lastSavedDate
						|| $date->getTimestamp() - $lastSavedDate->getTimestamp() > 300
						|| $session->isNew()){
					$session->save();
					$session->setTemp("lastSaved", $date);
					$session->setSessionChanged(false);
				}

				$mc = OZONE::$memcache;
				$key = 'session..'.$session->getSessionId();
				$mc->set($key, $session, 0, 600);
			}
        if(!empty($this->_outCookies)) {
            $this->_setCookies();
        }
	}

	/**
	 * Resets all the session data - i.e. stops a session and starts a new one.
	 */
	public function resetSession() {
		$this->sessionStop(false);
		$this->sessionStart();

	}

	public function sessionAdd($key, $value) {

		if ($this->session == null) {
			$this->sessionStart();
		}
		$this->session->setSerialized($key, $value);
	}

	public function sessionGet($key) {
		if ($this->session !== null) {
			return $this->session->getSerialized($key);
		} else{
			return null;
		}
	}

	public function sessionDel($key = null) {
		if ($this->session !== null) {
			$this->session->clearSerialized($key);
		}
	}

	public function clearSessionStorage($key = null) {
		if ($this->session !== null) {
			$this->session->clearSerialized($key);
		}
	}

	/**
	 * Returns an instance of the FormTool. FormTool requires usage of sessions!
	 */
	public function formTool() {
		$formTool = $this->sessionGet('form_tool');
		if ($formTool == null) {
			$formTool = new FormTool();
			$this->sessionAdd('form_tool', $formTool);
			OzoneLogger :: instance()->debug("obtaining new FormTool");
		}
		//
		if ($this->formToolHttpProcessed == false) {
			// extract form data form the http request
			$formTool->processHttpRequest($this);
			$this->formToolHttpProcessed = true;
		}

		return $formTool;

	}

	// SECURITY-RELATED METHODS FOLLOW:

	/**
	 * Checks if the current user is authenticated (registered + logged in) or
	 * anonymous. Returns true if authenticated, false otherwise.
	 */
	public function isUserAuthenticated() {
		$session = $this->session;
		if (!$session) {
			return false;
		}
		if(!$this->getUser()){
			return false;
		} else {
			return true;
		}
	}

	public function getUserId() {
		if($this->session == null){
			return null;
		}
		return $this->session->getUserId();
	}

	public function getOzoneUser(){
		return $this->getUser();
	}

	public function getUser() : ?User
    {
		if($this->session == null){
			return null;
		}
		return $this->session->getOzoneUser();
	}

	public function setExtra($key, $value){
		$this->extra[$key] = $value;
	}

	public function getExtra($key){
		return $this->extra[$key];
	}

	public function extraAsArray(){
		return $this->extra;
	}

	public function createIpString() {
	    # We'll revisit the need and viability of this approach when it's behind a load balancer.

//		if ($_SERVER["HTTP_X_FORWARDED_FOR"] && preg_match('/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$/', $_SERVER["HTTP_X_FORWARDED_FOR"]) === 1) {
//			if ($_SERVER["HTTP_CLIENT_IP"]) {
//				$proxy = $_SERVER["HTTP_CLIENT_IP"];
//			} else {
//				$proxy = $_SERVER["REMOTE_ADDR"];
//			}
//			$ip = $_SERVER["HTTP_X_FORWARDED_FOR"];
//			$out = $ip."|".$proxy;
//		} else {
//			if ($_SERVER["HTTP_CLIENT_IP"]) {
//				$ip = $_SERVER["HTTP_CLIENT_IP"];
//			} else {
//				$ip = $_SERVER["REMOTE_ADDR"];
//			}
//			$out = $ip;
//		}
//		return $out;

        # For now, there's no reason to trust the Client-IP or Forwarded-For headers, they can be arbitrarily set by the client.
		return $_SERVER["REMOTE_ADDR"];

	}

	public function createUaHash() {
		return md5($_SERVER['HTTP_USER_AGENT'] . GlobalProperties::$SECRET_LOGIN_SEED);
	}

	public function setTemp($key, $value){
		$this->temp[$key] = $value;
	}

	public function getTemp($key){
		return $this->temp[$key];
	}

	public function ajaxResponseAdd($key, $value){
		$this->ajaxResponse[$key] = $value;
	}

	public function ajaxResponseGet($key){
		return $this->ajaxResponse[$key];
	}

	public function getAjaxResponse(){
		return $this->ajaxResponse;
	}

	public function getSessionId(){
		if($this->session != null){
			return $this->session->getSessionId();
		} else {
			return null;
		}
	}

	protected function _setCookie($cookieName, $value = null, $time = null, $path = null, $domain = null){
		$this->_outCookies[$cookieName] = array(
		'value' => $value,
		'time' => $time,
		'path' => $path,
		'domain' => $domain
		);
	}

	protected function _setCookies(){
		foreach($this->_outCookies as $name => $cookie){
                setsecurecookie($name, $cookie['value'], $cookie['time'], $cookie['path'], $cookie['domain']);
		}
		return;
	}

}
