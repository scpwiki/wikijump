<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:19
         compiled from /var/www/wikidot/templates/modules/wiki/pagestagcloud/PagesTagCloudModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/modules/wiki/pagestagcloud/PagesTagCloudModule.tpl', 4, false),array('block', 't', '/var/www/wikidot/templates/modules/wiki/pagestagcloud/PagesTagCloudModule.tpl', 11, false),)), $this); ?>
<?php if ($this->_tpl_vars['tags']): ?>
	<div class="pages-tag-cloud-box">
		<?php if (count($_from = (array)$this->_tpl_vars['tags'])):
    foreach ($_from as $this->_tpl_vars['tag']):
?>
			<a class="tag" href="<?php echo $this->_tpl_vars['href'];  echo ((is_array($_tmp=$this->_tpl_vars['tag']['tag'])) ? $this->_run_mod_handler('escape', true, $_tmp, 'url') : smarty_modifier_escape($_tmp, 'url'));  if ($this->_tpl_vars['category']): ?>/category/<?php echo ((is_array($_tmp=$this->_tpl_vars['category']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp));  endif; ?>"
				style="font-size: <?php echo $this->_tpl_vars['tag']['size']; ?>
; color: rgb(<?php echo $this->_tpl_vars['tag']['color']['r']; ?>
, <?php echo $this->_tpl_vars['tag']['color']['g']; ?>
, <?php echo $this->_tpl_vars['tag']['color']['b']; ?>
);"
				><?php echo ((is_array($_tmp=$this->_tpl_vars['tag']['tag'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
		<?php endforeach; endif; unset($_from); ?>
	</div>
<?php else: ?>
	<p>
		<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>It seems you have no tags attached to pages. To attach a tag simply click on the <em>tags</em> 
		button at the bottom of any page.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
	</p>
<?php endif; ?>