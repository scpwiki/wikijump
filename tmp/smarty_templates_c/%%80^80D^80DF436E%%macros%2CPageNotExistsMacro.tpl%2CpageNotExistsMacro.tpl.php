<?php /* Smarty version 2.6.7, created on 2008-12-06 19:45:41
         compiled from /var/www/wikidot/tmp/smarty_macro_templates//macros%2CPageNotExistsMacro.tpl%2CpageNotExistsMacro.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/tmp/smarty_macro_templates//macros,PageNotExistsMacro.tpl,pageNotExistsMacro.tpl', 4, false),)), $this); ?>


<p>
<?php $this->_tag_stack[] = array('t', array('1' => $this->_tpl_vars['wikiPage'],'escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>The page <em>%1</em> you want to access does not exist.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
</p>
<ul>
	<li><a href="javascript:;" onclick="WIKIDOT.page.listeners.editClick(event)"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>create page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a></li>
</ul>

