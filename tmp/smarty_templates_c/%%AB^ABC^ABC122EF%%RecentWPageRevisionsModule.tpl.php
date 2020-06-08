<?php /* Smarty version 2.6.7, created on 2009-01-04 19:47:38
         compiled from /var/www/wikidot/templates/modules/wiki/sitesactivity/RecentWPageRevisionsModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/modules/wiki/sitesactivity/RecentWPageRevisionsModule.tpl', 26, false),array('block', 't', '/var/www/wikidot/templates/modules/wiki/sitesactivity/RecentWPageRevisionsModule.tpl', 32, false),)), $this); ?>
<div class="recent-w-page-revisions" id="recent-w-page-revisions">
	<?php if ($this->_tpl_vars['pages']): ?>
		<?php if (count($_from = (array)$this->_tpl_vars['pages'])):
    foreach ($_from as $this->_tpl_vars['page']):
?>
			<?php $this->assign('site', $this->_tpl_vars['page']->getSite()); ?>
			<div class="list-item">
				<div class="title">
					<a href="http://<?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getDomain())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
/<?php echo $this->_tpl_vars['page']->getUnixName(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['page']->getTitleOrUnixname())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
				</div>
				<div class="preview">
					<?php echo $this->_tpl_vars['page']->getPreview(100); ?>

				</div>
				<div class="in-site">
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>site<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>: <a href="http://<?php echo $this->_tpl_vars['site']->getDomain(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>, <br/><span class="odate"><?php echo $this->_tpl_vars['page']->getDateLastEdited()->getTimestamp(); ?>
|%O <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>ago<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></span>
				</div>
			</div>
		<?php endforeach; endif; unset($_from); ?>
	<?php endif; ?>

</div>