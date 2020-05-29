<?php /* Smarty version 2.6.7, created on 2020-04-09 13:53:17
         compiled from /var/www/wikidot/templates/modules/account/pm/PMInboxModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('function', 'pager', '/var/www/wikidot/templates/modules/account/pm/PMInboxModule.tpl', 3, false),array('function', 'printuser', '/var/www/wikidot/templates/modules/account/pm/PMInboxModule.tpl', 34, false),array('block', 't', '/var/www/wikidot/templates/modules/account/pm/PMInboxModule.tpl', 12, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/account/pm/PMInboxModule.tpl', 31, false),)), $this); ?>
<?php if ($this->_tpl_vars['totalPages'] > 1): ?>
	<div style="text-align: center">
		<?php echo smarty_function_pager(array('total' => $this->_tpl_vars['totalPages'],'current' => $this->_tpl_vars['currentPage'],'jsfunction' => 'inboxPage'), $this);?>

	</div>
<?php endif; ?>

<table class="pm-list">
	<tr class="headers">
		<td>&nbsp;</td>
		
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Subject<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Sender<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Date<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			&nbsp;
		</td>
	</tr>
	
	<?php if (count($_from = (array)$this->_tpl_vars['messages'])):
    foreach ($_from as $this->_tpl_vars['message']):
?>
		<tr>
			<td>&nbsp;</td>
			<td class="subject">
				<a href="javascript:;" 
				<?php if ($this->_tpl_vars['message']->getFlagNew()): ?> style="font-weight: bold"<?php endif; ?>
				onclick="WIKIDOT.modules.AccountMessagesModule.listeners.viewInboxMessage(<?php echo $this->_tpl_vars['message']->getMessageId(); ?>
)"><?php echo ((is_array($_tmp=$this->_tpl_vars['message']->getSubject())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
			</td>
			<td>
				<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['message']->getFromUser(),'image' => true), $this);?>

			</td>
			<td class="date">
				<span class="odate"><?php echo $this->_tpl_vars['message']->getDate()->getTimestamp(); ?>
|%e %b %Y, %H:%M %Z|agohover</span>
			</td>
			<td>
				<input class="message-select" type="checkbox" id="message-check-<?php echo $this->_tpl_vars['message']->getMessageId(); ?>
"/>
			</td>		
		</tr>
	<?php endforeach; endif; unset($_from); ?>
	
	<!-- options -->
	<?php if ($this->_tpl_vars['messages']): ?>
		<tr>	
			<td colspan="4"  style="padding: 1em 2em 0 0; text-align: right;">
				<a href="javascript:;" onclick="WIKIDOT.modules.PMInboxModule.listeners.removeSelected(event)"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>remove selected<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
			</td>
			<td  style="padding-top: 1em">
				[<a href="javascript:;" onclick="WIKIDOT.modules.PMInboxModule.listeners.selectAll(event)"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>select all<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>]
			</td>
		</tr>
	<?php endif; ?>
</table>

