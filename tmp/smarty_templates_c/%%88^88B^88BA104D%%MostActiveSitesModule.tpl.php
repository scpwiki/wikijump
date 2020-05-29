<?php /* Smarty version 2.6.7, created on 2009-01-04 19:47:39
         compiled from /var/www/wikidot/templates/modules/wiki/sitesactivity/MostActiveSitesModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/wiki/sitesactivity/MostActiveSitesModule.tpl', 4, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/wiki/sitesactivity/MostActiveSitesModule.tpl', 31, false),)), $this); ?>
<div class="most-active-sites-box">
	<div class="when">
		<?php if ($this->_tpl_vars['range'] == '24h'): ?>
			24 <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>hours<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		<?php else: ?>
			<a href="javascript:;" onclick="WIKIDOT.modules.MostActiveSitesModule.listeners.changeTime(event, '24h')"> 24 <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>hours<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
		<?php endif; ?>
		| 
		<?php if ($this->_tpl_vars['range'] == '7days'): ?>
			7 <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>days<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		<?php else: ?>
			<a href="javascript:;" onclick="WIKIDOT.modules.MostActiveSitesModule.listeners.changeTime(event, '7d')"> 7 <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>days<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
		<?php endif; ?>
		|
		<?php if ($this->_tpl_vars['range'] == 'month'): ?>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>month<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		<?php else: ?>
			<a href="javascript:;" onclick="WIKIDOT.modules.MostActiveSitesModule.listeners.changeTime(event, 'month')"> <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>month<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
		<?php endif; ?>
	</div>

	<?php if ($this->_tpl_vars['res']): ?>
		<table class="item-list">
			<tr>
				<td>&nbsp;</td>
				<td><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>changes<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></td>
			</tr>
			<?php if (count($_from = (array)$this->_tpl_vars['res'])):
    foreach ($_from as $this->_tpl_vars['r']):
?>
				<tr>
					<td>
						<a href="http://<?php echo $this->_tpl_vars['r']['site']->getDomain(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['r']['site']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
					</td>
					<td style="text-align: right">
						<?php echo $this->_tpl_vars['r']['number_changes']; ?>
 
					</td>
				</tr>
			<?php endforeach; endif; unset($_from); ?>
		</table>
	<?php else: ?>
		<p>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Sorry, no activity in this range.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</p>
	<?php endif; ?>

</div>