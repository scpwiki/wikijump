<?php

namespace Ozone\Framework;





/**
 * Cointains information required for page rendering, such as page title, css
 * style list, javascript files etc.
 */
class PageProperties{

	private $title = "No title";
	private $styles1 = array();
	private $styles2 = array();
	private $jsFiles = array();
	private $meta = array();
	private $httpEquivs = array();
	private $bodyProperties = array();
	private $links = array();
	private $skin = "default";

	private $layout = "Default";

	private $styleRaw1 = array();
	private $styleRaw2 = array();
	private $jsRaw = array();
	private $headRaw = array();

	private $styleSelector = 2;

	public function setStyleSelector($selector){
		$this->styleSelector = $selector;
	}

	public function addStyleSheet($file){ //, $title=null, $media=null, $type="text/css"){
		if($this->styleSelector == 1){
			$this->styles1[] = $file;
		} else if ($this->styleSelector == 2){
			$this->styles2[] = $file;
		}
	}

	public function getStyleSheets(){
		return array_merge($this->styles1, $this->styles2);
	}

	public function addStyleRaw($style){
		if($this->styleSelector == 1){
			$this->styleRaw1[] = $style;
		} else if ($this->styleSelector == 2){
			$this->styleRaw2[] = $style;
		}
	}

	public function getStyleRaw(){
		return array_merge($this->styleRaw1, $this->styleRaw2);
	}

	public function addJavaScriptRaw($js){
		$this->jsRaw[] = $js;
	}

	public function getJavaScriptRaw(){
		return $this->jsRaw;
	}

  public function addHeadRaw($raw){
    $this->headRaw[] = $raw;
  }

  public function getHeadRaw(){
    return $this->headRaw;
  }

  public function hasHeadRaw(){
    if(count($this->headRaw)>0){
      return true;
    } else {
      return false;
    }
  }

	public function hasJavaScriptRaw(){
		if(count($this->jsRaw)>0){
			return true;
		} else {
		 	return false;
		}
	}
	public function addJavaScript($file){
		$this->jsFiles[]=$file;
	}

	public function getJavaScripts(){
		return $this->jsFiles;
	}

	public function setLayout($layout){
		#replace just for sure...
		$this->layout = str_replace(',', '/',$layout);
	}

	public function getLayout(){
		return $this->layout;
	}

	public function getTitle(){
		return $this->title;
	}

	public function setTitle($title){
		$this->title = $title;
	}

	public function addMeta($name, $content){
		$this->meta[$name]=$content;
	}

	public function getMetas(){
		return $this->meta;
	}
	public function addHttpEquiv($equiv, $content){
		$this->httpEquivs[$equiv]=$content;
	}

	public function getHttpEquivs(){
		return $this->httpEquivs;
	}

	public function addLink($rel, $href, $type=null, $title=null){
		$this->links[] = array('rel' => $rel, 'href' => $href, 'type' => $type, 'title'=>$title);
	}

	public function getLinks(){
		return $this->links;
	}

	public function addBodyProperty($property, $value){
		$this->bodyProperties["$property"] = $value;
	}

	public function getBodyProperties(){
		return $this->bodyProperties;
	}

	public function getSkin(){
		return $this->skin;
	}

	public function setSkin($skin){
		$this->skin = $skin;
	}
}
