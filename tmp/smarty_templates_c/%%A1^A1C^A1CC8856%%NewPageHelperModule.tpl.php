<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:19
         compiled from /var/www/wikidot/templates/modules/misc/NewPageHelperModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/modules/misc/NewPageHelperModule.tpl', 4, false),array('block', 't', '/var/www/wikidot/templates/modules/misc/NewPageHelperModule.tpl', 6, false),)), $this); ?>

<div class="new-page-box" style="text-align: center; margin: 1em 0">
	<form action="dummy.html" method="get" onsubmit="WIKIDOT.modules.NewPageHelperModule.listeners.create(event)">
		<input class="text" name="pageName" type="text" size="<?php echo ((is_array($_tmp=$this->_tpl_vars['size'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
" maxlength="60" style="margin: 1px"/><?php if ($this->_tpl_vars['templates']): ?>
		<select name="template" style="margin: 1px">
			<option value="" selected="selected">-- <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Select a template<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> --</option>
			<?php if (count($_from = (array)$this->_tpl_vars['templates'])):
    foreach ($_from as $this->_tpl_vars['template']):
?>
				<option value="<?php echo $this->_tpl_vars['template']->getPageId(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['template']->getTitleOrUnixName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
			<?php endforeach; endif; unset($_from); ?>
		</select>
		<?php endif; ?><input type="submit" class="button" value="<?php if ($this->_tpl_vars['button']):  echo ((is_array($_tmp=$this->_tpl_vars['button'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp));  else:  $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>create page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack);  endif; ?>" style="margin: 1px"/>
		<?php if ($this->_tpl_vars['categoryName']): ?>
			<input type="hidden" name="categoryName" value="<?php echo $this->_tpl_vars['categoryName']; ?>
"/>
		<?php endif; ?>
		<?php if ($this->_tpl_vars['template']): ?>
			<input type="hidden" name="template" value="<?php echo $this->_tpl_vars['template']->getPageId(); ?>
"/>
		<?php endif; ?>
		<?php if ($this->_tpl_vars['format']): ?>
			<input type="hidden" name="format" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['format'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
"/>
		<?php endif; ?>
		<?php if ($this->_tpl_vars['autoincrement']): ?>
			<input type="hidden" name="autoincrement" value="true"/>
		<?php endif; ?>
		
	</form>
</div>
<?php if ($this->_tpl_vars['formatError']): ?>
	<div class="error-block">
		The format <?php echo ((is_array($_tmp=$this->_tpl_vars['formatError'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
 is not a valid regular expression in the NewPage module above.
	</div>
<?php endif; ?>