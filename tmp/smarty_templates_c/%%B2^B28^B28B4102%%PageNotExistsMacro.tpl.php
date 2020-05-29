<?php /* Smarty version 2.6.7, created on 2008-12-06 19:45:41
         compiled from /var/www/wikidot/templates/macros/PageNotExistsMacro.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 'defmacro', '/var/www/wikidot/templates/macros/PageNotExistsMacro.tpl', 2, false),)), $this); ?>

 <?php $this->_tag_stack[] = array('defmacro', array('name' => 'pageNotExistsMacro')); smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?> <?php echo ' 

<p>
{t 1=$wikiPage escape=no}The page <em>%1</em> you want to access does not exist.{/t}
</p>
<ul>
	<li><a href="javascript:;" onclick="WIKIDOT.page.listeners.editClick(event)">{t}create page{/t}</a></li>
</ul>

'; ?>
 <?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> 