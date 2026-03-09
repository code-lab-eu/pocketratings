import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import ManageListRow from '../../src/lib/ManageListRow.svelte';

describe('ManageListRow', () => {
  it('renders label as link when viewHref is set', () => {
    render(ManageListRow, {
      props: {
        label: 'Test Item',
        viewHref: '/view/1',
        editHref: '/edit/1',
        deleteLabel: 'Test Item',
        onDelete: () => {}
      }
    });
    const link = screen.getByRole('link', { name: 'Test Item' });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/view/1');
  });

  it('renders label as plain text when viewHref is not set', () => {
    render(ManageListRow, {
      props: {
        label: 'Plain Item',
        editHref: '/edit/2',
        deleteLabel: 'Plain Item',
        onDelete: () => {}
      }
    });
    expect(screen.getByText('Plain Item')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Plain Item' })).not.toBeInTheDocument();
  });

  it('renders Edit and Delete with accessible labels', () => {
    render(ManageListRow, {
      props: {
        label: 'Entity',
        editHref: '/edit/3',
        deleteLabel: 'Entity',
        onDelete: () => {}
      }
    });
    expect(screen.getByRole('link', { name: /edit entity/i })).toHaveAttribute('href', '/edit/3');
    expect(screen.getByRole('button', { name: /delete entity/i })).toBeInTheDocument();
  });

  it('calls onDelete when Delete is clicked', async () => {
    const onDelete = vi.fn();
    const user = userEvent.setup();
    render(ManageListRow, {
      props: {
        label: 'Item',
        editHref: '/edit/4',
        deleteLabel: 'Item',
        onDelete
      }
    });
    await user.click(screen.getByRole('button', { name: /delete item/i }));
    expect(onDelete).toHaveBeenCalledTimes(1);
  });

  it('disables Delete button when deleting is true', () => {
    render(ManageListRow, {
      props: {
        label: 'Item',
        editHref: '/edit/5',
        deleteLabel: 'Item',
        onDelete: () => {},
        deleting: true
      }
    });
    expect(screen.getByRole('button', { name: /delete item/i })).toBeDisabled();
  });
});
