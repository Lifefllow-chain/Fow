'use client';

import React from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeaderCell,
  TableRow,
} from '@/components/ui/Table';
import { Card, CardContent } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { format } from 'date-fns';

interface ReportingTableProps {
  data: any;
  isLoading?: boolean;
}

export function ReportingTable({ data, isLoading }: ReportingTableProps) {
  if (isLoading) {
    return (
      <div className="space-y-4">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="h-16 w-full bg-slate-100 animate-pulse rounded-xl" />
        ))}
      </div>
    );
  }

  const renderDonors = (donors: any[]) => (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold flex items-center gap-2">
        <span className="w-2 h-2 rounded-full bg-blue-500" /> Donors ({donors.length})
      </h3>
      <Table>
        <TableHead>
          <TableRow>
            <TableHeaderCell>Name</TableHeaderCell>
            <TableHeaderCell>Email</TableHeaderCell>
            <TableHeaderCell>Region</TableHeaderCell>
            <TableHeaderCell>Blood Type</TableHeaderCell>
            <TableHeaderCell>Joined</TableHeaderCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {donors.map((donor) => (
            <TableRow key={donor.id} className="hover:bg-slate-50 transition-colors">
              <TableCell className="font-medium">{donor.name || 'N/A'}</TableCell>
              <TableCell>{donor.email}</TableCell>
              <TableCell>{donor.region}</TableCell>
              <TableCell>
                <Badge variant="outline" className="bg-red-50 text-red-600 border-red-200">
                  {donor.profile?.bloodType || 'Unknown'}
                </Badge>
              </TableCell>
              <TableCell className="text-slate-500">
                {format(new Date(donor.createdAt), 'MMM d, yyyy')}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );

  const renderUnits = (units: any[]) => (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold flex items-center gap-2">
        <span className="w-2 h-2 rounded-full bg-red-500" /> Blood Units ({units.length})
      </h3>
      <Table>
        <TableHead>
          <TableRow>
            <TableHeaderCell>Unit Code</TableHeaderCell>
            <TableHeaderCell>Type</TableHeaderCell>
            <TableHeaderCell>Component</TableHeaderCell>
            <TableHeaderCell>Status</TableHeaderCell>
            <TableHeaderCell>Expires</TableHeaderCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {units.map((unit) => (
            <TableRow key={unit.id} className="hover:bg-slate-50 transition-colors">
              <TableCell className="font-mono font-bold text-blue-600">#{unit.unitCode}</TableCell>
              <TableCell>
                <Badge className="bg-red-600">{unit.bloodType}</Badge>
              </TableCell>
              <TableCell>{unit.component}</TableCell>
              <TableCell>
                <Badge variant={unit.status === 'AVAILABLE' ? 'default' : 'secondary'} className={unit.status === 'AVAILABLE' ? 'bg-green-600' : ''}>
                  {unit.status}
                </Badge>
              </TableCell>
              <TableCell className="text-slate-500">
                {format(new Date(unit.expiresAt), 'MMM d, yyyy')}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );

  const renderOrders = (orders: any[]) => (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold flex items-center gap-2">
        <span className="w-2 h-2 rounded-full bg-green-500" /> Market Orders ({orders.length})
      </h3>
      <Table>
        <TableHead>
          <TableRow>
            <TableHeaderCell>Order ID</TableHeaderCell>
            <TableHeaderCell>Blood Type</TableHeaderCell>
            <TableHeaderCell>Quantity</TableHeaderCell>
            <TableHeaderCell>Status</TableHeaderCell>
            <TableHeaderCell>Date</TableHeaderCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {orders.map((order) => (
            <TableRow key={order.id} className="hover:bg-slate-50 transition-colors">
              <TableCell className="font-mono text-xs">{order.id.slice(0, 8)}...</TableCell>
              <TableCell><Badge variant="outline">{order.bloodType}</Badge></TableCell>
              <TableCell>{order.quantity} units</TableCell>
              <TableCell>
                <Badge className={order.status === 'DELIVERED' ? 'bg-green-600' : 'bg-orange-500'}>
                  {order.status}
                </Badge>
              </TableCell>
              <TableCell className="text-slate-500">
                {format(new Date(order.createdAt), 'MMM d, yyyy')}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );

  return (
    <Card className="border-none shadow-2xl bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl">
      <CardContent className="p-8 space-y-12">
        {!data || (Array.isArray(data) && data.length === 0) ? (
          <div className="text-center py-20 text-slate-400">
            No records found. Adjust your filters to see results.
          </div>
        ) : (
          <>
            {data.donors && data.donors[0] && renderDonors(data.donors[0])}
            {data.units && data.units[0] && renderUnits(data.units[0])}
            {data.orders && data.orders[0] && renderOrders(data.orders[0])}
          </>
        )}
      </CardContent>
    </Card>
  );
}
