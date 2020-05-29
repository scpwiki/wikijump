<?php /* Smarty version 2.6.7, created on 2009-01-04 19:47:39
         compiled from /var/www/wikidot/templates/modules/wiki/sitesactivity/NewWUsersModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('function', 'printuser', '/var/www/wikidot/templates/modules/wiki/sitesactivity/NewWUsersModule.tpl', 6, false),array('block', 't', '/var/www/wikidot/templates/modules/wiki/sitesactivity/NewWUsersModule.tpl', 6, false),)), $this); ?>
<div class="new-w-users-box" id="new-w-users-box">
	<?php if ($this->_tpl_vars['users']): ?>
		<ul style="list-style: none; margin-left:0; padding-left:1em;">
			<?php if (count($_from = (array)$this->_tpl_vars['users'])):
    foreach ($_from as $this->_tpl_vars['user']):
?>
				<li>
					<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['user'],'image' => true), $this);?>
 <span class="odate" style="color: #777"><?php echo $this->_tpl_vars['user']->getRegisteredDate()->getTimestamp(); ?>
|%O <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>ago<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></span>
				</li>
			<?php endforeach; endif; unset($_from); ?>
		</ul>
	<?php endif; ?>
</div>