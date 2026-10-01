import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ProductDetailPage from '../../src/routes/products/[id]/+page.svelte';

const mocks = vi.hoisted(() => ({
  createReview: vi.fn().mockResolvedValue({ id: 'new-rev' }),
  createPurchase: vi.fn().mockResolvedValue({ id: 'new-pur' }),
  getProductVariations: vi.fn().mockResolvedValue([]),
  invalidateAll: vi.fn().mockResolvedValue(undefined)
}));

vi.mock('$lib/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/api')>();
  return {
    ...actual,
    createReview: mocks.createReview,
    createPurchase: mocks.createPurchase,
    getProductVariations: mocks.getProductVariations
  };
});

vi.mock('$app/navigation', () => ({
  invalidateAll: mocks.invalidateAll
}));
import type { PageData } from '../../src/routes/products/[id]/$types';
import type { Location, ProductDetail, Purchase, Review } from '../../src/lib/types';

const product: ProductDetail = {
  id: 'prod-1',
  category: { id: 'cat-1', name: 'Dairy', ancestors: [] },
  brand: 'Acme',
  name: 'Milk',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  variations: [
    { id: 'var-1', label: '', unit: 'none' },
    { id: 'var-2', label: '1 l', unit: 'milliliters', quantity: 1000 }
  ]
};

const location: Location = {
  id: 'loc-1',
  name: 'Store A',
  deleted_at: null
};

const review: Review = {
  id: 'rev-1',
  product: { id: 'prod-1', brand: 'Acme', name: 'Milk' },
  user: { id: 'u1', name: 'Alice' },
  rating: 4,
  text: 'Good product',
  created_at: 1000,
  updated_at: 1000,
  deleted_at: null
};

const purchase: Purchase = {
  id: 'pur-1',
  user: { id: 'u1', name: 'Alice' },
  product: { id: 'prod-1', brand: 'Acme', name: 'Milk' },
  variation: { id: 'var-1', label: '', unit: 'none' },
  location: { id: 'loc-1', name: 'Store A' },
  quantity: 1,
  price: '2.99',
  purchased_at: 1708012800,
  deleted_at: null
};

const defaultData: PageData = {
  product,
  reviews: [review],
  purchases: [purchase],
  locations: [location],
  notFound: false,
  error: null
};

/** Opens the inline add-purchase form and returns the purchase history region. */
async function openInlinePurchase() {
  render(ProductDetailPage, { props: { data: defaultData } });
  const region = screen.getByRole('region', { name: /^purchase history$/i });
  await userEvent.click(within(region).getByRole('button', { name: /add purchase/i }));
  return region;
}

describe('Product detail page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.createReview.mockResolvedValue({ id: 'new-rev' });
    mocks.createPurchase.mockResolvedValue({ id: 'new-pur' });
    mocks.invalidateAll.mockResolvedValue(undefined);
  });

  it('shows product name and brand; category link only in breadcrumb', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    expect(screen.getByRole('heading', { name: /milk/i })).toBeInTheDocument();
    const brand = document.querySelector('.pr-product-brand');
    expect(brand).toBeInTheDocument();
    expect(brand).toHaveTextContent(/^acme$/i);
    const dairyLinks = screen.getAllByRole('link', { name: /dairy/i });
    expect(dairyLinks).toHaveLength(1);
    expect(dairyLinks[0].getAttribute('href')).toContain('/categories/cat-1');
  });

  it('shows an edit link to the manage product page next to the product name', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const edit = screen.getByRole('link', { name: 'Edit Milk — Acme' });
    expect(edit.getAttribute('href')).toContain('/manage/products/prod-1');
    expect(edit.closest('header')).toContainElement(screen.getByRole('heading', { name: /milk/i }));
  });

  it('shows breadcrumb with Home, category, and product name when product is loaded', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    expect(nav).toBeInTheDocument();
    const homeLink = screen.getByRole('link', { name: /^home$/i });
    expect(homeLink.getAttribute('href')).toContain('/');
    const categoryLinkInBreadcrumb = nav.querySelector('a[href*="/categories/cat-1"]');
    expect(categoryLinkInBreadcrumb).toBeInTheDocument();
    expect(categoryLinkInBreadcrumb).toHaveTextContent(/dairy/i);
    const currentSegment = nav.querySelector('[aria-current="page"]');
    expect(currentSegment).toBeInTheDocument();
    expect(currentSegment).toHaveTextContent(/milk.*acme/i);
  });

  it('shows full breadcrumb with ancestors when product category has ancestors', () => {
    const productWithAncestors: ProductDetail = {
      ...product,
      category: {
        id: 'c2',
        name: 'Dairy',
        ancestors: [{ id: 'c1', name: 'Food' }]
      }
    };
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: productWithAncestors
        } as PageData
      }
    });
    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    const links = nav.querySelectorAll('a');
    expect(links).toHaveLength(3);
    expect(links[0]).toHaveAccessibleName('Home');
    expect(links[1]).toHaveAccessibleName('Food');
    expect(links[2]).toHaveAccessibleName('Dairy');
    const currentSegment = nav.querySelector('[aria-current="page"]');
    expect(currentSegment).toHaveTextContent(/milk.*acme/i);
  });

  it('does not mutate ancestors array across re-renders (breadcrumb order regression)', () => {
    const ancestors = [
      { id: 'dairy-id', name: 'Dairy' },
      { id: 'food-id', name: 'Food' }
    ];
    const productWithAncestors: ProductDetail = {
      ...product,
      category: { id: 'cheese-id', name: 'Cheese', ancestors }
    };
    const props = {
      data: { ...defaultData, product: productWithAncestors } as PageData
    };

    const { unmount } = render(ProductDetailPage, { props });
    unmount();
    render(ProductDetailPage, { props });

    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    const links = nav.querySelectorAll('a');
    expect(links).toHaveLength(4);
    expect(links[0]).toHaveAccessibleName('Home');
    expect(links[1]).toHaveAccessibleName('Food');
    expect(links[2]).toHaveAccessibleName('Dairy');
    expect(links[3]).toHaveAccessibleName('Cheese');

    expect(ancestors[0].name).toBe('Dairy');
    expect(ancestors[1].name).toBe('Food');
  });

  it('shows reviews section with rating, text, and user name', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    expect(screen.getByRole('heading', { name: /reviews/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/rating: 4\.0 out of 5/i)).toBeInTheDocument();
    expect(screen.getByText(/good product/i)).toBeInTheDocument();
    expect(screen.getByText(/by alice/i)).toBeInTheDocument();
  });

  it('shows purchase history with date, location, quantity, price (grouped by variation)', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    expect(screen.getByRole('heading', { name: /purchase history/i })).toBeInTheDocument();
    expect(screen.getByText(/store a/i)).toBeInTheDocument();
    expect(screen.getByText(/×1/)).toBeInTheDocument();
    expect(screen.getByText(/2\.99 €/)).toBeInTheDocument();
    expect(screen.getByText(/15 feb 2024/i)).toBeInTheDocument();
  });

  it('shows the quantity to the right of the price in purchase history', () => {
    render(ProductDetailPage, {
      props: { data: { ...defaultData, purchases: [{ ...purchase, quantity: 3 }] } as PageData }
    });
    const history = screen.getByRole('region', { name: /^purchase history$/i });
    const [row] = within(history).getAllByRole('listitem');
    expect(row.textContent?.replace(/\s+/g, ' ')).toMatch(/Store A 2\.99 € ×3$/);
  });

  it('shows quantity in purchase history when greater than one', () => {
    const purchaseQty2: Purchase = {
      ...purchase,
      id: 'pur-1',
      quantity: 2
    };
    render(ProductDetailPage, {
      props: { data: { ...defaultData, purchases: [purchaseQty2] } as PageData }
    });
    expect(screen.getByText(/×2/)).toBeInTheDocument();
  });

  it('orders purchase history with most recent first', () => {
    const older: Purchase = {
      ...purchase,
      id: 'pur-1',
      purchased_at: 1704067200,
      location: { id: 'loc-1', name: 'Store A' }
    };
    const newer: Purchase = {
      ...purchase,
      id: 'pur-2',
      purchased_at: 1709251200,
      location: { id: 'loc-2', name: 'Store B' }
    };
    render(ProductDetailPage, {
      props: {
        data: { ...defaultData, purchases: [older, newer] } as PageData
      }
    });
    const section = screen.getByRole('heading', { name: /purchase history/i }).closest('section');
    const listItems = section!.querySelectorAll('ul li');
    expect(listItems).toHaveLength(2);
    expect(listItems[0]).toHaveTextContent(/store b/i);
    expect(listItems[0]).toHaveTextContent(/1 mar 2024/i);
    expect(listItems[1]).toHaveTextContent(/store a/i);
    expect(listItems[1]).toHaveTextContent(/1 jan 2024/i);
  });

  it('shows variation sub-headings when multiple variations have purchases', () => {
    const purchaseDefault: Purchase = {
      ...purchase,
      id: 'pur-1',
      variation: { id: 'var-1', label: '', unit: 'none' }
    };
    const purchase500g: Purchase = {
      ...purchase,
      id: 'pur-2',
      variation: { id: 'var-2', label: '500 g', unit: 'grams' },
      location: { id: 'loc-2', name: 'Store B' }
    };
    render(ProductDetailPage, {
      props: {
        data: { ...defaultData, purchases: [purchaseDefault, purchase500g] } as PageData
      }
    });
    expect(screen.getByRole('heading', { name: /default/i, level: 3 })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /500 g/i, level: 3 })).toBeInTheDocument();
  });

  it('shows Add review in reviews section and Add purchase at the bottom of purchase history', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const reviewsRegion = screen.getByRole('region', { name: /^reviews$/i });
    expect(within(reviewsRegion).getByRole('button', { name: /add review/i })).toBeInTheDocument();

    const historyRegion = screen.getByRole('region', { name: /^purchase history$/i });
    const addPurchase = within(historyRegion).getByRole('button', { name: /add purchase/i });
    const lastPurchase = within(historyRegion).getAllByRole('listitem').at(-1)!;
    expect(
      lastPurchase.compareDocumentPosition(addPurchase) & Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy();

    expect(screen.queryByRole('region', { name: /^actions$/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /add purchase/i })).not.toBeInTheDocument();
  });

  it('shows Add purchase when there are no purchases yet', () => {
    render(ProductDetailPage, {
      props: { data: { ...defaultData, purchases: [] } as PageData }
    });
    const historyRegion = screen.getByRole('region', { name: /^purchase history$/i });
    expect(within(historyRegion).getByRole('button', { name: /add purchase/i })).toBeInTheDocument();
  });

  it('opens inline add-purchase form for this product with the default variation selected', async () => {
    const region = await openInlinePurchase();

    const form = within(region).getByRole('form', { name: /add purchase/i });
    expect(within(form).queryByLabelText(/^product$/i)).not.toBeInTheDocument();
    const variation = within(form).getByLabelText<HTMLSelectElement>(/variation/i);
    expect(variation.value).toBe('var-1');
    expect(within(variation).getAllByRole('option').map((o) => o.textContent)).toEqual([
      'Select variation',
      'Default',
      '1 l'
    ]);
    const locationSelect = within(form).getByLabelText<HTMLSelectElement>(/location/i);
    expect(within(locationSelect).getByRole('option', { name: 'Store A' })).toBeInTheDocument();
    expect(within(form).getByLabelText(/quantity/i)).toHaveValue(1);
    expect(within(form).getByLabelText('Price per item (EUR)')).toHaveValue('');
    expect(within(form).getByLabelText<HTMLInputElement>(/date/i).value).not.toBe('');
    expect(within(region).queryByRole('button', { name: /add purchase/i })).not.toBeInTheDocument();
    expect(mocks.getProductVariations).not.toHaveBeenCalled();
  });

  it('closes inline add-purchase form on Cancel without saving', async () => {
    const region = await openInlinePurchase();

    await userEvent.click(within(region).getByRole('button', { name: /^cancel$/i }));

    expect(within(region).queryByRole('form', { name: /add purchase/i })).not.toBeInTheDocument();
    expect(within(region).getByRole('button', { name: /add purchase/i })).toBeInTheDocument();
    expect(mocks.createPurchase).not.toHaveBeenCalled();
  });

  it('submits inline purchase for this product then invalidates all and closes the form', async () => {
    const region = await openInlinePurchase();
    const form = within(region).getByRole('form', { name: /add purchase/i });

    await userEvent.selectOptions(within(form).getByLabelText(/variation/i), 'var-2');
    await userEvent.selectOptions(within(form).getByLabelText(/location/i), 'loc-1');
    await userEvent.clear(within(form).getByLabelText(/quantity/i));
    await userEvent.type(within(form).getByLabelText(/quantity/i), '2');
    await userEvent.type(within(form).getByLabelText(/price/i), '3.49');
    await userEvent.click(within(form).getByRole('button', { name: /^save$/i }));

    expect(mocks.createPurchase).toHaveBeenCalledWith(
      expect.objectContaining({
        product_id: 'prod-1',
        variation_id: 'var-2',
        location_id: 'loc-1',
        quantity: 2,
        price: '3.49',
        purchased_at: expect.any(String)
      })
    );
    expect(mocks.createPurchase).toHaveBeenCalledBefore(mocks.invalidateAll);
    expect(mocks.invalidateAll).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(within(region).getByRole('button', { name: /add purchase/i })).toBeInTheDocument();
    });
    expect(within(region).queryByRole('form', { name: /add purchase/i })).not.toBeInTheDocument();
  });

  it('requires a variation in the inline purchase form', async () => {
    const region = await openInlinePurchase();
    const form = within(region).getByRole('form', { name: /add purchase/i });
    const variation = within(form).getByLabelText<HTMLSelectElement>(/variation/i);
    expect(variation).toBeRequired();

    await userEvent.selectOptions(variation, '');
    await userEvent.selectOptions(within(form).getByLabelText(/location/i), 'loc-1');
    await userEvent.type(within(form).getByLabelText(/price/i), '3.49');
    await userEvent.click(within(form).getByRole('button', { name: /^save$/i }));

    expect(variation.validity.valueMissing).toBe(true);
    expect(mocks.createPurchase).not.toHaveBeenCalled();
  });

  it('keeps inline purchase form open with an error when price is missing', async () => {
    const region = await openInlinePurchase();
    const form = within(region).getByRole('form', { name: /add purchase/i });

    await userEvent.selectOptions(within(form).getByLabelText(/location/i), 'loc-1');
    await userEvent.click(within(form).getByRole('button', { name: /^save$/i }));

    expect(within(form).getByText('Price is required.')).toBeInTheDocument();
    expect(mocks.createPurchase).not.toHaveBeenCalled();
  });

  it('keeps inline purchase form open and shows the API error when saving fails', async () => {
    mocks.createPurchase.mockRejectedValue(new Error('Location not found.'));
    const region = await openInlinePurchase();
    const form = within(region).getByRole('form', { name: /add purchase/i });

    await userEvent.selectOptions(within(form).getByLabelText(/location/i), 'loc-1');
    await userEvent.type(within(form).getByLabelText(/price/i), '3.49');
    await userEvent.click(within(form).getByRole('button', { name: /^save$/i }));

    expect(await within(form).findByText('Location not found.')).toBeInTheDocument();
    expect(mocks.invalidateAll).not.toHaveBeenCalled();
    expect(within(region).getByRole('form', { name: /add purchase/i })).toBeInTheDocument();
  });

  it('opens inline add-review form and closes on Cancel', async () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const reviewsRegion = screen.getByRole('region', { name: /^reviews$/i });
    await userEvent.click(within(reviewsRegion).getByRole('button', { name: /add review/i }));

    expect(screen.getByRole('button', { name: /^save$/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /^cancel$/i })).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: /^cancel$/i }));

    expect(screen.queryByRole('button', { name: /^save$/i })).not.toBeInTheDocument();
    expect(within(reviewsRegion).getByRole('button', { name: /add review/i })).toBeInTheDocument();
    expect(mocks.createReview).not.toHaveBeenCalled();
  });

  it('submits inline review then invalidates all so data can refresh', async () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const reviewsRegion = screen.getByRole('region', { name: /^reviews$/i });
    await userEvent.click(within(reviewsRegion).getByRole('button', { name: /add review/i }));
    await userEvent.click(screen.getByRole('button', { name: /^save$/i }));

    expect(mocks.createReview).toHaveBeenCalledBefore(mocks.invalidateAll);
    expect(mocks.createReview).toHaveBeenCalledWith(
      expect.objectContaining({ product_id: 'prod-1', rating: 3 })
    );
    expect(mocks.invalidateAll).toHaveBeenCalledTimes(1);

    await waitFor(() => {
      expect(within(reviewsRegion).getByRole('button', { name: /add review/i })).toBeInTheDocument();
    });
    expect(screen.queryByRole('button', { name: /^save$/i })).not.toBeInTheDocument();
  });

  it('shows Not found and back link when notFound is true', () => {
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: null,
          reviews: [],
          purchases: [],
          notFound: true,
          error: null
        } as unknown as PageData
      }
    });
    expect(screen.getByText(/product not found/i)).toBeInTheDocument();
    expect(screen.queryByRole('navigation', { name: 'Breadcrumb' })).not.toBeInTheDocument();
    const backLink = screen.getByRole('link', { name: /back to home/i });
    expect(backLink).toBeInTheDocument();
    expect(backLink.getAttribute('href')).toContain('/');
  });

  it('shows error when error is set', () => {
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: null,
          reviews: [],
          purchases: [],
          notFound: false,
          error: 'Not found'
        } as PageData
      }
    });
    expect(screen.getByText('Not found')).toBeInTheDocument();
    expect(screen.queryByRole('navigation', { name: 'Breadcrumb' })).not.toBeInTheDocument();
    const homeLink = screen.getByRole('link', { name: /home/i });
    expect(homeLink.getAttribute('href')).toContain('/');
  });

  it('shows product not found when product is null and no error and not notFound', () => {
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: null,
          reviews: [],
          purchases: [],
          notFound: false,
          error: null
        } as unknown as PageData
      }
    });
    expect(screen.getByText(/product not found/i)).toBeInTheDocument();
  });

  it('shows empty reviews message when no reviews', () => {
    render(ProductDetailPage, {
      props: { data: { ...defaultData, reviews: [] } as PageData }
    });
    expect(screen.getByText(/no reviews yet/i)).toBeInTheDocument();
  });

  it('shows empty purchase history message when no purchases', () => {
    render(ProductDetailPage, {
      props: { data: { ...defaultData, purchases: [] } as PageData }
    });
    expect(screen.getByText(/this pocket is empty/i)).toBeInTheDocument();
  });
});
