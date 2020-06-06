<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:19
         compiled from /var/www/wikidot/templates/layouts/WikiLayout.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/layouts/WikiLayout.tpl', 7, false),array('modifier', 'regex_replace', '/var/www/wikidot/templates/layouts/WikiLayout.tpl', 91, false),array('modifier', 'replace', '/var/www/wikidot/templates/layouts/WikiLayout.tpl', 184, false),array('function', 'module', '/var/www/wikidot/templates/layouts/WikiLayout.tpl', 42, false),array('function', 'macro', '/var/www/wikidot/templates/layouts/WikiLayout.tpl', 126, false),array('block', 't', '/var/www/wikidot/templates/layouts/WikiLayout.tpl', 85, false),)), $this); ?>
<!DOCTYPE html 
     PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
     "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="<?php echo $this->_tpl_vars['site']->getLanguage(); ?>
" lang="<?php echo $this->_tpl_vars['site']->getLanguage(); ?>
">

<head>
 	<title><?php echo $this->_tpl_vars['site']->getName();  if ($this->_tpl_vars['wikiPage'] && $this->_tpl_vars['wikiPage']->getTitle()): ?>: <?php echo ((is_array($_tmp=$this->_tpl_vars['wikiPage']->getTitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp));  endif; ?></title>
 	<script type="text/javascript" src="/common--javascript/json.js"></script>
 	
	<script type="text/javascript" src="/common--javascript/combined.js"></script>
	
 	<script type="text/javascript" src="/common--javascript/OZONE.js"></script>
 	<script type="text/javascript" src="/common--javascript/dialog/OZONE.dialog.js"></script>
 	
 	<script  type="text/javascript">
 		// global request information
 		<?php echo '
 		var WIKIREQUEST = {};
 		WIKIREQUEST.info = {};
 		'; ?>

 		WIKIREQUEST.info.domain = "<?php echo $this->_tpl_vars['site']->getDomain(); ?>
";
 		WIKIREQUEST.info.siteId = <?php echo $this->_tpl_vars['site']->getSiteId(); ?>
;
 		WIKIREQUEST.info.categoryId = <?php echo $this->_tpl_vars['category']->getCategoryId(); ?>
;
 		WIKIREQUEST.info.themeId = <?php echo $this->_tpl_vars['theme']->getThemeId(); ?>
;
 		WIKIREQUEST.info.requestPageName = "<?php echo $this->_tpl_vars['wikiPageName']; ?>
";
 		OZONE.request.timestamp = %%%CURRENT_TIMESTAMP%%%;
 		OZONE.request.date = new Date();
 		WIKIREQUEST.info.lang = '<?php echo $this->_tpl_vars['site']->getLanguage(); ?>
';
 		<?php if ($this->_tpl_vars['wikiPage']): ?>
 		WIKIREQUEST.info.pageUnixName = "<?php echo $this->_tpl_vars['wikiPage']->getUnixName(); ?>
";
 		WIKIREQUEST.info.pageId = <?php echo $this->_tpl_vars['wikiPage']->getPageId(); ?>
;
 		<?php endif; ?>
 		WIKIREQUEST.info.lang = "<?php echo $this->_tpl_vars['site']->getLanguage(); ?>
";
 		OZONE.lang = "<?php echo $this->_tpl_vars['site']->getLanguage(); ?>
";
// 		window.onload = WikidotInit();

		var HTTP_SCHEMA = '<?php echo $this->_tpl_vars['HTTP_SCHEMA']; ?>
		var URL_HOST = '<?php echo $this->_tpl_vars['URL_HOST']; ?>
';
		var URL_DOMAIN = '<?php echo $this->_tpl_vars['URL_DOMAIN']; ?>
';
 	</script>
 	

 	<?php echo smarty_function_module(array('name' => "login/CustomDomainScriptModule"), $this);?>

	<?php echo smarty_function_module(array('name' => "login/FileAuthScriptModule"), $this);?>


 	
 	<meta http-equiv="content-type" content="text/html;charset=UTF-8"/>
    <meta http-equiv="content-language" content="<?php echo $this->_tpl_vars['site']->getLanguage(); ?>
"/>
 	
 	<script type="text/javascript" src="/common--javascript/WIKIDOT.js"></script>
 	<script type="text/javascript" src="/common--javascript/WIKIDOT.page.js"></script>
 	<script type="text/javascript" src="/common--javascript/WIKIDOT.editor.js"></script>
 	
   	<style type="text/css" id="internal-style">
   		
   		<?php if (count($_from = (array)$this->_tpl_vars['theme']->getStyleUrls())):
    foreach ($_from as $this->_tpl_vars['file']):
?>
   			@import url(<?php echo $this->_tpl_vars['file']; ?>
);
   		<?php endforeach; endif; unset($_from); ?>
   		
    </style>
    
    <link rel="shortcut icon" href="/common--theme/base/images/favicon.gif"/>
    <link rel="icon" type="image/gif" href="/common--theme/base/images/favicon.gif"/>
    
        <?php if ($this->_tpl_vars['openId'] && $this->_tpl_vars['openId']['enabled']): ?>
    	<link rel="openid.server" href="<?php echo $this->_tpl_vars['openId']['server']; ?>
" />
  		<link rel="openid.delegate" href="<?php echo $this->_tpl_vars['openId']['identity']; ?>
" />
  		<meta http-equiv="X-XRDS-Location" content="<?php echo $this->_tpl_vars['openId']['identity']; ?>
/xrds" />
    <?php endif; ?>
    
</head>

  <body id="html-body">

	<div id="container-wrap">
		<div id="container">
		  	<div id="header">
		  		<h1><a href="/"><span><?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</span></a></h1>
		  		<?php if ($this->_tpl_vars['site']->getSubtitle()): ?>
		  			<h2><span><?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getSubtitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</span></h2>
		  		<?php endif; ?>
		  		
		  		<div id="search-top-box">
		  			<form id="search-top-box-form" action="dummy">
			  			<input id="search-top-box-input" class="text empty" type="text" size="15" name="query" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>search this wiki<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onfocus="<?php echo 'if(YAHOO.util.Dom.hasClass(this, \'empty\')){YAHOO.util.Dom.removeClass(this,\'empty\'); this.value=\'\';}'; ?>
"/><input class="button" type="submit" name="search" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>search<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>"/>
				 	</form>
	  			</div>
		  		
		  		<?php if ($this->_tpl_vars['topBarContent']): ?>
			  		<div id="top-bar">
			  			<?php echo ((is_array($_tmp=$this->_tpl_vars['topBarContent'])) ? $this->_run_mod_handler('regex_replace', true, $_tmp, "/>\s+</s", "><") : smarty_modifier_regex_replace($_tmp, "/>\s+</s", "><")); ?>

			  		</div>
		  		<?php endif; ?>
		  		<div id="login-status"><?php echo $this->_tpl_vars['module']->render('LoginStatusModule'); ?>
</div>
		  		<div id="header-extra-div-1"><span></span></div><div id="header-extra-div-2"><span></span></div><div id="header-extra-div-3"><span></span></div>
		  	</div>
		  	
			<div id="content-wrap">
				<?php if ($this->_tpl_vars['sideBar1Content']): ?>
					<div id="side-bar">
						<?php echo ((is_array($_tmp=$this->_tpl_vars['sideBar1Content'])) ? $this->_run_mod_handler('regex_replace', true, $_tmp, "/>\s+</s", "><") : smarty_modifier_regex_replace($_tmp, "/>\s+</s", "><")); ?>

					</div>
				<?php endif; ?>
				
				<div id="main-content">
					<div id="action-area-top"></div>
					
					<?php if ($this->_tpl_vars['wikiPage'] == null || $this->_tpl_vars['wikiPage']->getTitle() != ''): ?>
					<div id="page-title">
					<?php if ($this->_tpl_vars['wikiPage']):  echo ((is_array($_tmp=$this->_tpl_vars['wikiPage']->getTitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp));  else:  $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>The page does not (yet) exist.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack);  endif; ?>
					</div>
					<?php endif; ?>
					<?php if ($this->_tpl_vars['breadcrumbs']): ?>
						<div id="breadcrumbs">
							<?php if (count($_from = (array)$this->_tpl_vars['breadcrumbs'])):
    foreach ($_from as $this->_tpl_vars['breadcrumb']):
?>
								<a href="/<?php echo $this->_tpl_vars['breadcrumb']->getUnixName(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['breadcrumb']->getTitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a> &raquo;
							<?php endforeach; endif; unset($_from); ?>
							<?php echo ((is_array($_tmp=$this->_tpl_vars['wikiPage']->getTitleOrUnixName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>

						</div>
					<?php endif; ?>
					
					
					<div id="page-content">
						<?php if ($this->_tpl_vars['pageNotExists']): ?>
							<?php echo $this->_tpl_vars['macros']->load('PageNotExistsMacro'); ?>

							<?php echo smarty_function_macro(array('name' => 'pageNotExistsMacro','wikiPage' => $this->_tpl_vars['wikiPageName']), $this);?>

						<?php endif; ?>
						<?php echo $this->_tpl_vars['screen_placeholder']; ?>

					</div>
					<?php if ($this->_tpl_vars['tags']): ?>
						<div class="page-tags">
							<span>
								<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>page tags<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
								<?php if (count($_from = (array)$this->_tpl_vars['tags'])):
    foreach ($_from as $this->_tpl_vars['tag']):
?>
									<a href="/system:page-tags/tag/<?php echo ((is_array($_tmp=$this->_tpl_vars['tag'])) ? $this->_run_mod_handler('escape', true, $_tmp, 'url') : smarty_modifier_escape($_tmp, 'url')); ?>
#pages"><?php echo ((is_array($_tmp=$this->_tpl_vars['tag'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
								<?php endforeach; endif; unset($_from); ?>
							</span>
						</div>
					<?php endif; ?>
					
					<div style="clear:both; height:1px; font-size:1px;"></div>
					<?php if (! $this->_tpl_vars['pageNotExists']): ?>
						<?php echo smarty_function_module(array('name' => 'PageOptionsBottomModule','showDiscuss' => $this->_tpl_vars['category']->getShowDiscuss(),'threadId' => $this->_tpl_vars['wikiPage']->getThreadId(),'pageUnixName' => $this->_tpl_vars['wikiPage']->getUnixName()), $this);?>
			
					<?php endif; ?>
					
					<div id="action-area" style="display: none"></div>
				</div>
			</div>
			
			
				 		<div id="footer">
	 			<div class="options">
		 			<a href="<?php echo $this->_tpl_vars['URL_DOCS']; ?>
" id="wikidot-help-button">
			 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>help<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
			 		</a>
		 					 			|
			 		<a href="javascript:;" id="bug-report-button" 
			 			onclick="WIKIDOT.page.listeners.pageBugReport(event)">
			 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>report a bug<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
			 		</a>
		 			|
			 		<a href="javascript:;" id="abuse-report-button" 
			 			onclick="WIKIDOT.page.listeners.flagPageObjectionable(event)">
			 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>flag as objectionable<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
			 		</a>
			 		
	 			</div>
	 			Part of <a href="http://<?php echo $this->_tpl_vars['URL_HOST']; ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['SERVICE_NAME'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
	 			&#8212; 
 				Powered by <a href="http://www.wikidot.org/">Wikidot</a>
	 		</div>
	 		<?php if ($this->_tpl_vars['licenseText'] != ""): ?>
		 		<div id="license-area" class="license-area">
		 			<?php echo ((is_array($_tmp=$this->_tpl_vars['licenseText'])) ? $this->_run_mod_handler('replace', true, $_tmp, '%%UNLESS%%', 'Unless stated otherwise Content of this page is licensed under') : smarty_modifier_replace($_tmp, '%%UNLESS%%', 'Unless stated otherwise Content of this page is licensed under')); ?>

				</div>
			<?php endif; ?>
			
			<div id="extrac-div-1"><span></span></div><div id="extrac-div-2"><span></span></div><div id="extrac-div-3"><span></span></div>
			
	 	</div>
	 </div>
 	
 	<!-- These extra divs/spans may be used as catch-alls to add extra imagery. -->
	<div id="extra-div-1"><span></span></div><div id="extra-div-2"><span></span></div><div id="extra-div-3"><span></span></div>
	<div id="extra-div-4"><span></span></div><div id="extra-div-5"><span></span></div><div id="extra-div-6"><span></span></div>
 	
 	 	
 	<div id="page-options-bottom-tips" style="display: none">
 		<div id="edit-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Click here to edit contents of this page.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 	</div>
 	<div id="page-options-bottom-2-tips"  style="display: none">
 		<div id="edit-sections-button-hovertip">
	 		<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Click here to toggle editing of individual sections of the page (if possible).
	 		Watch headings for an "edit" link when available.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="edit-append-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Append content without editing the whole page source.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="history-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Check out how this page has evolved in the past.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="discuss-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>If you want to discuss contents of this page - this is the easiest way to do it.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="files-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>View and manage file attachments for this page.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="site-tools-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>A few useful tools to manage this Site.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="backlinks-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>	See pages that link to and include this page.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="rename-move-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Change the name (also URL address, possibly the category) of the page.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="view-source-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>View wiki source for this page without editing.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="parent-page-button-hovertip">	
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>View/set parent page (used for creating breadcrumbs and structured layout).<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		
 		<div id="abuse-report-button-hovertip">
 			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>	Notify administrators if there is objectionable content in this page.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		<div id="bug-report-button-hovertip">
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Something does not work as expected? Find out what you can do.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
 		</div>
 		
 	</div>
 	
 	<div id="account-notifications-dummy" style="display:none"></div>
 	
 	<div style="display:none" id="dummy-ondomready-block"></div>
  </body>

</html>