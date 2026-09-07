'use client';

import React from 'react';
import { Card, CardContent } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/label';
import { Select } from '@/components/ui/Select';
import { Button } from '@/components/ui/Button';
import { Download, Search } from 'lucide-react';

interface FilterPanelProps {
  onSearch: (filters: any) => void;
  onExport: () => void;
  isLoading?: boolean;
}

const DOMAIN_OPTIONS = [
  { value: 'all', label: 'Global Search' },
  { value: 'donors', label: 'Donors' },
  { value: 'units', label: 'Blood Units' },
  { value: 'orders', label: 'Orders' },
  { value: 'disputes', label: 'Disputes' },
  { value: 'organizations', label: 'Organizations' },
];

const BLOOD_TYPE_OPTIONS = [
  { value: 'all', label: 'All Types' },
  ...['A+', 'A-', 'B+', 'B-', 'O+', 'O-', 'AB+', 'AB-'].map((t) => ({
    value: t,
    label: t,
  })),
];

export function FilterPanel({ onSearch, onExport, isLoading }: FilterPanelProps) {
  const [filters, setFilters] = React.useState({
    startDate: '',
    endDate: '',
    domain: 'all',
    status: '',
    bloodType: '',
    location: '',
  });

  const handleChange = (key: string, value: string) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <Card className="mb-8">
      <CardContent className="p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div className="space-y-2">
            <Label>Date Range</Label>
            <div className="flex gap-2">
              <Input
                type="date"
                value={filters.startDate}
                onChange={(e) => handleChange('startDate', e.target.value)}
              />
              <Input
                type="date"
                value={filters.endDate}
                onChange={(e) => handleChange('endDate', e.target.value)}
              />
            </div>
          </div>

          <Select
            label="Search Domain"
            options={DOMAIN_OPTIONS}
            value={filters.domain}
            onChange={(e) => handleChange('domain', e.target.value)}
          />

          <Select
            label="Blood Type"
            placeholder="All types"
            options={BLOOD_TYPE_OPTIONS}
            value={filters.bloodType}
            onChange={(e) => handleChange('bloodType', e.target.value)}
          />

          <div className="space-y-2">
            <Label>Location / Region</Label>
            <Input
              placeholder="Filter by city or region..."
              value={filters.location}
              onChange={(e) => handleChange('location', e.target.value)}
            />
          </div>
        </div>

        <div className="mt-8 flex justify-end gap-3">
          <Button variant="outline" onClick={onExport} disabled={isLoading}>
            <Download className="h-4 w-4" /> Export Report
          </Button>
          <Button onClick={() => onSearch(filters)} disabled={isLoading}>
            <Search className="h-4 w-4" /> {isLoading ? 'Searching...' : 'Search Records'}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
